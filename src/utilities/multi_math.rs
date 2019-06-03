extern crate cgmath;

use std::f32;
use std::ops::{Add, Mul, Neg, Div, Sub};
use utilities::simd::{
    SimdFloat4, SimdFloat8
};
use utilities::math;
use self::cgmath::Vector3;

pub trait Sqrt {
    fn op_sqrt(&self) -> Self;
}

impl Sqrt for f32 {
    fn op_sqrt(&self) -> Self {
        self.sqrt()
    }
}

pub trait Vec3OpsElem
    where Self : Mul<Output = Self> + Div<Output = Self> + Add<Output = Self> + Sub<Output = Self>,
          Self : Neg<Output = Self>,
          Self : Sqrt + Sized
{}

impl Vec3OpsElem for f32 {}
impl Vec3OpsElem for SimdFloat4 {}
impl Vec3OpsElem for SimdFloat8 {}

// TODO test inline
pub trait Vec3Ops<Elem: Vec3OpsElem + Copy> : Sized {
    fn create(x: Elem, y: Elem, z: Elem) -> Self;
    fn get_x(&self) -> Elem;
    fn get_y(&self) -> Elem;
    fn get_z(&self) -> Elem;

    fn op_dot(&self, other: &Self) -> Elem {
        self.get_x() * other.get_x() +
            self.get_y() * other.get_y() +
            self.get_z() * other.get_z()
    }

    fn op_magnitude(&self) -> Elem {
        self.op_dot(self).op_sqrt()
    }

    fn op_normalized(self) -> Self {
        let magnitude = self.op_magnitude();
        self.op_divby_scalar(magnitude)
    }

    fn op_cross(self, other: Self) -> Self {
        Self::create(
            self.get_y() * other.get_z() - self.get_z() * other.get_y(),
            self.get_z() * other.get_x() - self.get_x() * other.get_z(),
            self.get_x() * other.get_y() - self.get_y() * other.get_x()
        )
    }

    fn op_neg(self) -> Self {
        Self::create(-self.get_x(), -self.get_y(), -self.get_z())
    }

    fn op_divby_scalar(self, other: Elem) -> Self {
        self.op_apply_elemwise(|x| x / other)
    }

    fn op_apply_elemwise<F: Fn(Elem) -> Elem>(self, f: F) -> Self {
        Self::create(f(self.get_x()), f(self.get_y()), f(self.get_z()))
    }
}

impl<T: Vec3OpsElem + Copy> Vec3Ops<T> for cgmath::Vector3<T>
{
    fn create(x: T, y: T, z: T) -> Self {
        cgmath::Vector3::new(x, y, z)
    }

    fn get_x(&self) -> T {
        self.x
    }

    fn get_y(&self) -> T {
        self.y
    }

    fn get_z(&self) -> T {
        self.z
    }
}

pub trait MultiNum {
    type Scalar: Clone;
    type Vector3: Clone;
    type UnitVector3: Clone;

    fn scalar_zero() -> Self::Scalar;
    fn scalar_inf() -> Self::Scalar;
    fn scalar_big_epsilon() -> Self::Scalar;
}

pub struct MultiNum1;
impl MultiNum for MultiNum1 {
    type Scalar = f32;
    type Vector3 = cgmath::Vector3<Self::Scalar>;
    type UnitVector3 = math::UnitVector3<Self::Scalar>;

    fn scalar_zero() -> f32 { 0.0f32 }
    fn scalar_inf() -> f32 { f32::INFINITY }
    fn scalar_big_epsilon() -> f32 { 0.00003 }
}

#[cfg(target_feature = "sse2")]
pub struct MultiNum4;
#[cfg(target_feature = "sse2")]
impl MultiNum for MultiNum4 {
    type Scalar = SimdFloat4;
    type Vector3 = cgmath::Vector3<Self::Scalar>;
    type UnitVector3 = math::UnitVector3<Self::Scalar>;

    fn scalar_zero() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_zero()) }
    fn scalar_inf() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_inf()) }
    fn scalar_big_epsilon() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_big_epsilon()) }
}

#[cfg(target_feature = "avx")]
pub struct MultiNum8;
#[cfg(target_feature = "avx")]
impl MultiNum for MultiNum8 {
    type Scalar = SimdFloat8;
    type Vector3 = cgmath::Vector3<Self::Scalar>;
    type UnitVector3 = math::UnitVector3<Self::Scalar>;

    fn scalar_zero() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_zero()) }
    fn scalar_inf() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_inf()) }
    fn scalar_big_epsilon() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_big_epsilon()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    use utilities::math::{Vec3, apprx_eq_vec3};

    #[test]
    fn test_cross() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);

        let test_c = a.op_cross(b);
        let expected_c = Vec3::new(0.0, 0.0, 1.0);

        assert!(apprx_eq_vec3(test_c, expected_c, f32::EPSILON));
    }
}