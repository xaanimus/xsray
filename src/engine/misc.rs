
extern crate cgmath;
use std::ops::Neg;
use std::ops::Range;
use std::f32;

pub type Vec3 = cgmath::Vector3<f32>;
pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Color3 = Vec3;

pub use self::cgmath::{Zero, SquareMatrix, InnerSpace};

fn f32_to_u8_color(x: f32) -> u8 {
    f32::max(0f32, f32::min(x * 255f32, 255f32)) as u8
}

///A Vec3 that's always normalized
#[derive(Debug)]
pub struct UnitVec3 {
    value: Vec3
}

impl UnitVec3 {
    pub fn new(value: &Vec3) -> UnitVec3 {
        UnitVec3 {
            value: value.normalize()
        }
    }

    pub fn vec(&self) -> &Vec3 {
        &self.value
    }

    pub fn cross(&self, other: UnitVec3) -> UnitVec3 {
        self.value.cross(other.value).unit()
    }

    pub fn clone(&self) -> UnitVec3 {
        UnitVec3 {
            value: self.value.clone()
        }
    }
}

impl Neg for UnitVec3 {
    type Output = UnitVec3;
    fn neg(self) -> UnitVec3 {
        UnitVec3 {
            value: -self.value
        }
    }
}

pub trait HasUnit<T> {
    fn unit(&self) -> T;
}

///This converts Vec3 into a unit version of Vec3.
///The converted value's type guarantees that it will have a magnitude of 1
impl HasUnit<UnitVec3> for Vec3 {
    fn unit(&self) -> UnitVec3 {
        UnitVec3::new(&self.clone()) //TODO check if this works without clone
    }
}

pub struct RayBase<T> {
    pub position: Vec3,
    pub direction: T,
    pub t_range: Range<f32>,
}

pub type RayUnit = RayBase<UnitVec3>;

impl<T> RayBase<T> {
    pub fn new(position: Vec3, direction: T) -> RayBase<T> {
        RayBase {
            position: position,
            direction: direction,
            t_range: 0.0..(f32::INFINITY)
        }
    }

    pub fn new_shadow(position: Vec3, direction: T) -> RayBase<T> {
        let mut ray = RayBase::<T>::new(position, direction);
        ray.t_range.start = 10.0 * f32::EPSILON;
        ray
    }
}

pub trait PixelRgb8Extractable {
    fn pixel_rgb8_values(&self) -> (u8, u8, u8);
}

impl PixelRgb8Extractable for Color3 {
    fn pixel_rgb8_values(&self) -> (u8, u8, u8) {
        (f32_to_u8_color(self.x),
         f32_to_u8_color(self.y),
         f32_to_u8_color(self.z))
    }
}
