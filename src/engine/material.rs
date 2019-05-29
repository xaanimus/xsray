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
use utilities::sampler::Sampler;
use utilities::sampler::NumberSequenceSampler;

pub enum Material {
    BSDF(Box<BSDFMaterial>)
}

pub fn default_material() -> Material {
    unimplemented!();
}

pub struct LightDirectionPair<'a> {
    pub incoming: &'a UnitVec3,
    pub outgoing: &'a UnitVec3
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum DeserializableMaterialSpec {
    Diffuse { color: CodableWrapper<Color3> },
    Microfacet { color: CodableWrapper<Color3>, ior: f32, roughness: f32}
}
impl_deserialize!(CodableWrapper<Rc<Material>>, |deserializer| {
    use self::DeserializableMaterialSpec::*;
    let material_spec = DeserializableMaterialSpec::deserialize(deserializer)?;
    let material_ptr: Rc<Material> = match material_spec {
        Diffuse {color} =>
            Rc::new(DiffuseBSDFMaterial::new(color.get()).into()),
        Microfacet { color, ior, roughness} =>
            Rc::new(MicrofacetReflectiveBSDFMaterial::new(ior, roughness, color.get()).into())
    };
    Ok(CodableWrapper(material_ptr))
});

struct BSDFSampleResult {
    sample: UnitVec3,
    pdf: f32
}
pub trait BSDFMaterial: Debug {
    fn sample(
        &self, normal: &UnitVec3, outgoing_light_direction: &UnitVec3,
        sampler: &mut NumberSequenceSampler
    ) -> BSDFSampleResult;

    fn sample_pdf(
        &self, normal: &UnitVec3, light_directions: &LightDirectionPair
    ) -> f32;

    fn brdf(&self, normal: &UnitVec3, light_directions: &LightDirectionPair) -> Color3;
}
impl<T> From<T> for Material
    where T: BSDFMaterial + 'static
{
    fn from(bsdf_material: T) -> Self {
        Material::BSDF(Box::new(bsdf_material))
    }
}

#[derive(Debug)]
pub struct DiffuseBSDFMaterial {
    color: Color3
}
impl DiffuseBSDFMaterial {
    pub fn new(color: Color3) -> DiffuseBSDFMaterial {
        DiffuseBSDFMaterial { color }
    }
}
impl BSDFMaterial for DiffuseBSDFMaterial {
    fn sample(
        &self, normal: &UnitVec3, outgoing_light_direction: &UnitVec3,
        sampler: &mut NumberSequenceSampler
    ) -> BSDFSampleResult {
        let untransformed_sample = CosineHemisphereWarper.sample(sampler);
        let sample = transform_into(normal, &untransformed_sample);

        let pdf = {
            let light_directions = LightDirectionPair {
                incoming: &sample,
                outgoing: outgoing_light_direction
            };
            self.sample_pdf(normal, &light_directions)
        };

        BSDFSampleResult {
            sample,
            pdf
        }
    }

    fn sample_pdf(&self, normal: &UnitVec3, light_directions: &LightDirectionPair) -> f32 {
        let sample = transform_from(normal, light_directions.incoming.value());
        CosineHemisphereWarper.pdf(sample.value())
    }

    fn brdf(&self, normal: &UnitVec3, light_directions: &LightDirectionPair) -> Vec3 {
        if light_directions.outgoing.value().dot(*normal.value()) < 0.0 {
            return Color3::zero()
        }

        self.color / PI
    }
}

#[derive(Debug)]
pub struct MicrofacetReflectiveBSDFMaterial {
    index_of_refraction: f32,
    roughness: f32,
    color: Color3,
    warper: GGXNormalHalfVectorWarper
}
impl MicrofacetReflectiveBSDFMaterial {
    fn new(index_of_refraction: f32, roughness: f32, color: Color3) -> MicrofacetReflectiveBSDFMaterial {
        MicrofacetReflectiveBSDFMaterial {
            index_of_refraction: index_of_refraction,
            roughness: roughness,
            color: color,
            warper: GGXNormalHalfVectorWarper {
                alpha: roughness
            }
        }
    }
}
impl BSDFMaterial for MicrofacetReflectiveBSDFMaterial {
    fn sample(
        &self, normal: &UnitVec3, outgoing_light_direction: &UnitVec3,
        sampler: &mut NumberSequenceSampler
    ) -> BSDFSampleResult {
        let half_vector = transform_into(
            normal, &GGXNormalHalfVectorWarper { alpha: self.roughness }.sample(sampler)
        );
        let incoming_light_direction = reflection(outgoing_light_direction, &half_vector);
        //incoming_light_direction
        unimplemented!()
    }

    fn sample_pdf(&self, normal: &UnitVec3, light_directions: &LightDirectionPair) -> f32 {
        let half = half_vector(light_directions.incoming, light_directions.outgoing);
        ggx_distribution(&half, normal, self.roughness) * normal.value().dot(*half.value()).abs()
    }

    fn brdf(&self, normal: &UnitVec3, light_directions: &LightDirectionPair) -> Vec3 {
        let alpha = self.roughness;
        let ior = self.index_of_refraction;
        let f0 = fresnel_schlick_at_normal(ior);
        let half = half_vector(light_directions.incoming, light_directions.outgoing);
        let num = fresnel_schlick(light_directions.incoming, &half, f0) *
            ggx_distribution(&half, normal, alpha) *
            ggx_geometry(light_directions, &half, normal, alpha);

        let denom =
            light_directions.incoming.value().dot(*normal.value()).abs() *
                light_directions.outgoing.value().dot(*normal.value()).abs() * 4.0;

        let result = self.color * num * normal.value().dot(*light_directions.incoming.value())
            / denom;
        result.max_elem_wise(&Color3::zero())
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

fn ggx_geometry(
    light_directions: &LightDirectionPair,
    half_vector: &UnitVec3,
    normal: &UnitVec3,
    alpha: f32
) -> f32 {
    let a2 = alpha.powi(2);
    let n = *normal.value();
    let v = *light_directions.outgoing.value();
    let m = *half_vector.value();
    let theta_viewing = v.dot(n).acos();

    let numer = chi_plus(v.dot(m) / v.dot(n)) * 2.0;
    let denom = 1.0 +
        (1.0 + a2 * theta_viewing.tan().powi(2)).sqrt();

    numer / denom
}

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

fn fresnel_schlick_at_normal(index_of_refraction: f32) -> f32 {
    let n = index_of_refraction;
    ((n - 1.0) / (n + 1.0)).powi(2)
}

fn fresnel_schlick(
    incoming_light_direction: &UnitVec3,
    half_vector: &UnitVec3,
    normal_reflectance: f32
) -> f32 {
    let f0 = normal_reflectance;
    let wi = incoming_light_direction.value();
    let m = half_vector.value();
    f0 + (1.0 - f0) * ((1.0 - wi.dot(*m)).powi(5))
}
