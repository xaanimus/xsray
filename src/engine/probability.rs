extern crate cgmath;
extern crate rand;

use self::cgmath::Rad;

use utilities::math::*;
use std::f32::consts::PI;
use std::f32;
use utilities::sampler::Sampler;

pub trait Warper {
    type Output;

    fn sample<Spl: Sampler>(&self, sampler: &mut Spl) -> Self::Output {
        let (x, y) = sampler.get_2d_f32();
        let input = Vec2 {x, y};
        self.warp(&input)
    }

    ///Warps a 2d uniform random sample into the output
    fn warp(&self, from: &Vec2) -> Self::Output;

    ///Gives the probability density for sampling the given output
    ///This assumes the output is inside the range of the warp function.
    ///In other words, output should not only be of type Self::Output,
    ///but should also have been returned from warp
    fn pdf(&self, output: &Self::Output) -> f32;
}

///Warper for a unit circle
pub struct UniformCircleWarper;
impl Warper for UniformCircleWarper {
    type Output = Vec2;

    fn warp(&self, from: &Vec2) -> Self::Output {
        let theta = 2.0 * PI * from.x;
        let radius = from.y.sqrt();
        Vec2 {
            x: radius * theta.cos(),
            y: radius * theta.sin()
        }
    }

    fn pdf(&self, _: &Self::Output) -> f32 {
        1.0 / PI
    }
}

///Warper for a unit hemisphere
pub struct UniformHemisphereWarper;
impl Warper for UniformHemisphereWarper {
    type Output = Vec3;

    fn warp(&self, from: &Vec2) -> Self::Output {
        let height = from.x;
        let theta = 2.0 * PI * from.y;
        let r = (1.0 - height.powi(2)).sqrt();
        Vec3 {
            x: r * theta.cos(),
            y: height,
            z: - r * theta.sin()
        }
    }

    fn pdf(&self, _: &Self::Output) -> f32 {
        1.0 / (2.0 * PI)
    }
}


///Warper for a unit hemisphere
pub struct UniformSphereWarper;
impl Warper for UniformSphereWarper {
    type Output = Vec3;

    fn warp(&self, from: &Vec2) -> Self::Output {
        let height = from.x * 2.0 - 1.0;
        let theta = 2.0 * PI * from.y;
        let r = (1.0 - height.powi(2)).sqrt();
        Vec3 {
            x: r * theta.cos(),
            y: height,
            z: - r * theta.sin()
        }
    }

    fn pdf(&self, _: &Self::Output) -> f32 {
        1.0 / (4.0 * PI)
    }
}

pub struct CosineHemisphereWarper;
impl Warper for CosineHemisphereWarper {
    type Output = Vec3;

    fn warp(&self, from: &Vec2) -> Self::Output {
        let height = from.x;
        let theta = 2.0 * PI * from.y;
        let r = (1.0 - height.powi(2)).sqrt();
        Vec3 {
            x: r * theta.cos(),
            y: height,
            z: - r * theta.sin()
        }
    }

    fn pdf(&self, sample: &Self::Output) -> f32 {
        let normal = Vec3::new(0., 1., 0.);
        sample.dot(normal).max(0.0) / PI
    }
}

/// Samples half vectors for ggx
pub struct GGXNormalHalfVectorWarper {
    pub alpha: f32
}
impl Warper for GGXNormalHalfVectorWarper {
    type Output = Vec3;

    fn warp(&self, from: &Vec2) -> Self::Output {
        //xi is like a fancy squiggly E
        let xi = from.x;
        let phi = Rad(2.0 * PI * from.y);
        let theta = Rad(
            ( self.alpha * (xi / (1.0 - xi)).sqrt() ).atan()
        );
        let rotation =
            Matrix3::from_angle_y(phi) *
            Matrix3::from_angle_z(theta);
        rotation * Vec3::new(0.0, 1.0, 0.0)
    }

    fn pdf(&self, sample: &Self::Output) -> f32 {
        let normal = Vec3::new(0.0, 1.0, 0.0).unit();
        ggx_distribution(&sample.unit(), &normal, self.alpha) * normal.value().dot(*sample)
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

///TODO better doc
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

pub fn ggx_distribution(half_vector: &UnitVec3, normal: &UnitVec3, alpha: f32) -> f32 {
    let a2 = alpha.powi(2);
    let n = *normal.value();
    let m = *half_vector.value();
    let denom = PI * { n.dot(m).powi(2) * (a2 - 1.0) + 1.0 }.powi(2);
    a2 / denom
}

//this is broken for some reason. don't know why.
//pub fn ggx_distribution(half_vector: &UnitVec3, normal: &UnitVec3, alpha: f32) -> f32 {
//    let a2 = alpha.powi(2);
//    let n = *normal.value();
//    let m = *half_vector.value();
//    let theta = n.dot(m).acos();
//
//    let numer = a2 * chi_plus(n.dot(m));
//    let denom = PI *
//        n.dot(m).powi(4) *
//        (a2 + f32::tan(theta).powi(2)).powi(2);
//
//    numer / denom //0, .08, 1.2
//}

//TODO try to do this without branching
pub fn chi_plus(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else {
        1.0
    }
}

