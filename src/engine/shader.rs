use super::intersectable::IntersectionRecord;
use super::scene::{Scene};
use super::math::*;
use super::color::*;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::f32;

pub trait Shader {
    fn shade(&self, record: &IntersectionRecord, scene: &Scene) -> Color3;
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
}
