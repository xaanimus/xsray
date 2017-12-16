extern crate cgmath;
pub use std::ops::{Neg, Range};
use std::f32;

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Matrix3 = cgmath::Matrix3<f32>;

pub use self::cgmath::{Zero, One, Array, SquareMatrix, InnerSpace, ElementWise};

pub fn apprx_eq(a: f32, b: f32, eps: f32) -> bool {
    let x = (b - a).abs();
    x < eps
}

impl From<UnitVec3> for Vec3 {
    fn from(uvec: UnitVec3) -> Self {
        uvec._value
    }
}

///A Vec3 that's always normalized
///TODO implement into Vec3
#[derive(Debug, Clone)]
pub struct UnitVec3 {
    _value: Vec3
}

impl UnitVec3 {
    pub fn new(value: &Vec3) -> UnitVec3 {
        UnitVec3 {
            _value: value.normalize()
        }
    }

    pub fn value(&self) -> &Vec3 {
        &self._value
    }

    pub fn cross(&self, other: UnitVec3) -> UnitVec3 {
        self._value.cross(other._value).unit()
    }

    pub fn clone(&self) -> UnitVec3 {
        UnitVec3 {
            _value: self._value.clone()
        }
    }
}

impl Neg for UnitVec3 {
    type Output = UnitVec3;
    fn neg(self) -> UnitVec3 {
        UnitVec3 {
            _value: -self._value
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

#[derive(Clone)]
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

    pub fn new_epsilon_offset(position: Vec3, direction: T) -> RayBase<T> {
        let mut ray = RayBase::<T>::new(position, direction);
        ray.t_range.start = 100.0 * f32::EPSILON;
        ray
    }
}

pub trait HasElemWiseExtrema {
    fn min_elem_wise(&self, other: &Self) -> Self;
    fn max_elem_wise(&self, other: &Self) -> Self;
}

impl HasElemWiseExtrema for Vec3 {
    fn min_elem_wise(&self, other: &Vec3) -> Vec3{
        Vec3::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    fn max_elem_wise(&self, other: &Vec3) -> Vec3{
        Vec3::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }
}
