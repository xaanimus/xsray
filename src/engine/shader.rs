extern crate serde;
use self::serde::de::Error;
use utilities::codable::*;
use utilities::math::*;

use super::intersectable::IntersectionRecord;
use super::scene::{Scene};
use super::color::*;
use super::probability::*;

use std::fmt::{Debug, Formatter};
use std::fmt;
use std::f32;
use std::f32::consts::PI;
use std::rc::Rc;

pub fn default_shader() -> DiffuseShader {
    DiffuseShader {
        color: Color3::new(1.0, 1.0, 1.0)
    }
}

pub struct LightDirectionPair<'a> {
    pub incoming: &'a UnitVec3,
    pub outgoing: &'a UnitVec3
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum DeserializableShaderSpec {
    Diffuse { color: CodableWrapper<Color3> },
    Microfacet { color: CodableWrapper<Color3>, ior: f32, roughness: f32}
}

impl_deserialize!(CodableWrapper<Rc<Shader>>, |deserializer| {
    use self::DeserializableShaderSpec::*;
    let shader_spec = DeserializableShaderSpec::deserialize(deserializer)?;
    let shader_ptr: Rc<Shader> = match shader_spec {
        Diffuse {color} => Rc::new(DiffuseShader::new(color.get())),
        Microfacet { color, ior, roughness} => Rc::new(
            MicrofacetReflectiveShader {
                index_of_refraction: ior,
                roughness: roughness,
                color: color.get()
            }
        )
    };
    Ok(CodableWrapper(shader_ptr))
});

pub trait Shader {
    //TODO get rid of shade function since it's not used anymore
    fn shade(&self, record: &IntersectionRecord, scene: &Scene) -> Color3 {
        Color3::zero()
    }
    fn sample_bounce(&self, normal: &UnitVec3, outgoing_light_direction: &UnitVec3) -> UnitVec3;
    fn probability_of_sample(&self, normal: &UnitVec3,
                             light_directions: &LightDirectionPair) -> f32;
    fn brdf_cosine_term(
        &self, normal: &UnitVec3, light_directions: &LightDirectionPair
    ) -> Color3;
}

impl Debug for Shader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Shader")
    }
}

pub struct DiffuseShader {
    color: Color3
}

impl DiffuseShader {
    pub fn new(color: Color3) -> DiffuseShader {
        DiffuseShader {
            color: color
        }
    }
}

impl Shader for DiffuseShader {
    fn sample_bounce(&self, normal: &UnitVec3, outgoing_light_direction: &UnitVec3) -> UnitVec3 {
        let sample = CosineHemisphereWarper.sample();
        transform_into(normal, &sample)
    }

    fn probability_of_sample(&self, normal: &UnitVec3,
                             light_directions: &LightDirectionPair) -> f32 {
        let sample = transform_from(normal, light_directions.incoming.value());
        CosineHemisphereWarper.pdf(sample.value())
    }

    fn shade(&self, record: &IntersectionRecord, scene: &Scene) -> Color3 {
        scene.lights.iter().fold(Color3::new(0.0, 0.0, 0.0), |acc, light| {
            let light_vec = light.position.get() - record.position;
            //see if there is an obstruction to this light
            if scene.intersect_for_obstruction(record.position, light.position.get())
                .t < f32::INFINITY {
                acc
            } else {
                f32::max(0., record.normal.dot(light_vec.normalize())) *
                    self.color * light.intensity / light_vec.magnitude2() + acc
            }
        })
    }

    fn brdf_cosine_term(
        &self, normal: &UnitVec3, light_directions: &LightDirectionPair
    ) -> Color3 {
        let brdf = self.color / PI;
        let cosine_term = normal.value().dot(*light_directions.incoming.value()); //TODO figure out situation with Into
        brdf * cosine_term
    }
}

pub struct MicrofacetReflectiveShader {
    index_of_refraction: f32,
    roughness: f32,
    color: Color3
}

impl Shader for MicrofacetReflectiveShader {
    fn sample_bounce(&self, normal: &UnitVec3, outgoing_light_direction: &UnitVec3) -> UnitVec3 {
        //let sample = UniformHemisphereWarper.sample();
        //transform_into(normal, &sample)

        let half_vector = transform_into(
            normal, &GGXNormalHalfVectorWarper { alpha: self.roughness }.sample()
        );
        let incoming_light_direction = reflection(outgoing_light_direction, &half_vector);
        incoming_light_direction
    }

    fn probability_of_sample(&self, normal: &UnitVec3,
                             light_directions: &LightDirectionPair) -> f32 {
        //UniformHemisphereWarper.pdf(light_directions.incoming.value())
        let half = half_vector(light_directions.incoming, light_directions.outgoing);
        distribution_ggx(&half, normal, self.roughness) * normal.value().dot(*half.value())
    }

    fn brdf_cosine_term(
        &self, normal: &UnitVec3, light_directions: &LightDirectionPair
    ) -> Color3 {
        let alpha = self.roughness;
        let ior = self.index_of_refraction;
        let f0 = fresnel_normal_reflectance(ior);
        let half = half_vector(light_directions.incoming, light_directions.outgoing);
        let num = fresnel_term(light_directions.outgoing, &half, f0) *
            distribution_ggx(&half, normal, alpha) *
            geometry_neumann(light_directions, &half, normal);
        let denom =
            light_directions.incoming.value().dot(*normal.value()).abs() *
            light_directions.outgoing.value().dot(*normal.value()).abs() * 4.0;
        self.color * num * normal.value().dot(*light_directions.incoming.value())
            / denom
    }
}

fn half_vector(a: &UnitVec3, b: &UnitVec3) -> UnitVec3 {
    (a.value() + b.value()).unit()
}

fn reflection(light_outgoing: &UnitVec3, normal: &UnitVec3) -> UnitVec3 {
    let wo = *light_outgoing.value();
    let n = *normal.value();
    { -wo + n * (2.0 * wo.dot(n)) }.unit()
}

// TODO replace with probability::distribution_ggx
//fn distribution_ggx(half_vector: &UnitVec3, normal: &UnitVec3, alpha: f32) -> f32 {
//    let a2 = alpha.powi(2);
//    let n = *normal.value();
//    let m = *half_vector.value();
//    let denom = PI * { n.dot(m).powi(2) * (a2 - 1.0) + 1.0 }.powi(2);
//    a2 / denom
//}

fn geometry_neumann(
    light_directions: &LightDirectionPair,
    half_vector: &UnitVec3,
    normal: &UnitVec3
) -> f32 {
    let wi = *light_directions.incoming.value();
    let wo = *light_directions.outgoing.value();
    let _h = *half_vector.value();
    let n = *normal.value();
    let num = n.dot(wi) * n.dot(wo);
    let denom = n.dot(wi).max(n.dot(wo));
    num / denom
}

fn fresnel_normal_reflectance(index_of_refraction: f32) -> f32 {
    let n = index_of_refraction;
    ((1.0 - n) / (1.0 + n)).powi(2)
}

fn fresnel_term(
    outgoing_light_direction: &UnitVec3,
    half_vector: &UnitVec3,
    normal_reflectance: f32
) -> f32 {
    let f0 = normal_reflectance;
    let w_o = outgoing_light_direction.value();
    let m = half_vector.value();
    (1.0 - f0) * (1.0 - w_o.dot(*m)).powi(5) + f0
}
