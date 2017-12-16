extern crate rand;

use utilities::math::*;
use std::f32::consts::PI;

pub trait Warper {
    type Output;

    fn sample() -> Self::Output {
        let input = Vec2 {
            x: rand::random::<f32>(),
            y: rand::random::<f32>()
        };
        Self::warp(&input)
    }

    ///Warps a 2d uniform random sample into the output
    fn warp(from: &Vec2) -> Self::Output;

    ///Gives the probability density for sampling the given output
    ///This assumes the output is inside the range of the warp function.
    ///In other words, output should not only be of type Self::Output,
    ///but should also have been returned from warp
    fn pdf(output: &Self::Output) -> f32;
}

///Warper for a unit circle
enum UniformCircleWarper {}
impl Warper for UniformCircleWarper {
    type Output = Vec2;

    fn warp(from: &Vec2) -> Self::Output {
        let theta = 2.0 * PI * from.x;
        let radius = from.y.sqrt();
        Vec2 {
            x: radius * theta.cos(),
            y: radius * theta.sin()
        }
    }

    fn pdf(_: &Self::Output) -> f32 {
        1.0 / PI
    }
}

///Warper for a unit hemisphere
pub enum UniformHemisphereWarper {}
impl Warper for UniformHemisphereWarper {
    type Output = Vec3;

    fn warp(from: &Vec2) -> Self::Output {
        let height = from.x;
        let theta = 2.0 * PI * from.y;
        let r = (1.0 - height.powi(2)).sqrt();
        Vec3 {
            x: r * theta.cos(),
            y: height,
            z: - r * theta.sin()
        }
    }

    fn pdf(_: &Self::Output) -> f32 {
        1.0 / (2.0 * PI)
    }
}


///Warper for a unit hemisphere
enum UniformSphereWarper {}
impl Warper for UniformSphereWarper {
    type Output = Vec3;

    fn warp(from: &Vec2) -> Self::Output {
        let height = from.x * 2.0 - 1.0;
        let theta = 2.0 * PI * from.y;
        let r = (1.0 - height.powi(2)).sqrt();
        Vec3 {
            x: r * theta.cos(),
            y: height,
            z: - r * theta.sin()
        }
    }

    fn pdf(_: &Self::Output) -> f32 {
        1.0 / (4.0 * PI)
    }
}

pub enum CosineHemisphereWarper {}
impl Warper for CosineHemisphereWarper {
    type Output = Vec3;

    fn warp(from: &Vec2) -> Self::Output {
        let height = from.x;
        let theta = 2.0 * PI * from.y;
        let r = (1.0 - height.powi(2)).sqrt();
        Vec3 {
            x: r * theta.cos(),
            y: height,
            z: - r * theta.sin()
        }
    }

    fn pdf(sample: &Self::Output) -> f32 {
        let normal = Vec3::new(0., 1., 0.);
        sample.dot(normal).max(0.0) / PI
    }
}


fn get_rotation_matrix_to(normal: &UnitVec3) -> Matrix3 {
    let axis_0 = { //axis that's perpendicular to normal
        let up = Vec3::new(0.0, 1.0, 0.0);
        let x_dir = Vec3::new(1.0, 0.0, 0.0);
        let dot = up.dot(*normal.value());
        if dot.abs() < 0.95 {
            up.cross(*normal.value()).normalize()
        } else {
            x_dir.cross(*normal.value()).normalize()
        }
    };
    let axis_1 = axis_0.cross(*normal.value());

    Matrix3 {
        x: axis_0,
        y: *normal.value(),
        z: axis_1
    }
}

///Rotates the sample from [0,1,0] to the normal
pub fn transform_into(normal: &UnitVec3, sample: &Vec3) -> UnitVec3 {
    let rot_matrix = get_rotation_matrix_to(normal);
    (rot_matrix * *sample).unit()
}

///Rotates the sample from normal to [0,1,0]
pub fn transform_from(normal: &UnitVec3, sample: &Vec3) -> UnitVec3 {
    //TODO check that the rotation matrix is always invertible and handle this better if not
    let rot_matrix = get_rotation_matrix_to(normal)
        .invert().unwrap_or(Matrix3::one());
    (rot_matrix * *sample).unit()
}
