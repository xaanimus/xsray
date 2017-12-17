use super::intersectable::IntersectionRecord;
use super::scene::{Scene};
use super::math::*;
use super::color::*;
use super::probability::*;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::f32;
use std::f32::consts::PI;

pub fn default_shader() -> DiffuseShader {
    DiffuseShader {
        color: Color3::new(1.0, 1.0, 1.0)
    }
}

pub struct LightDirectionPair<'a> {
    pub incoming: &'a UnitVec3,
    pub outgoing: &'a UnitVec3
}

pub trait Shader {
    fn shade(&self, record: &IntersectionRecord, scene: &Scene) -> Color3;
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
        let sample = CosineHemisphereWarper::sample();
        transform_into(normal, &sample)
    }

    fn probability_of_sample(&self, normal: &UnitVec3,
                             light_directions: &LightDirectionPair) -> f32 {
        let sample = transform_from(normal, light_directions.incoming.value());
        CosineHemisphereWarper::pdf(sample.value())
    }

    fn shade(&self, record: &IntersectionRecord, scene: &Scene) -> Color3 {
        scene.lights.iter().fold(Color3::new(0.0, 0.0, 0.0), |acc, light| {
            let light_vec = light.position - record.position;
            //see if there is an obstruction to this light
            if scene.intersect_for_obstruction(record.position, light.position).t < f32::INFINITY {
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

