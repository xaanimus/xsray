extern crate cgmath;

use std::f32;
use std::ops::{Add, Mul, Neg, Div, Sub};
use utilities::cmp_util::CmpFn;
use utilities::simd::{SimdFloat4, SimdFloat8, Align16, Align32};
use utilities::math;
use self::cgmath::Vector3;
use utilities::math::{apprx_eq, Vec3};

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
pub trait Vec3Ops<Elem: Vec3OpsElem + Copy> : Sized + Copy {
    fn create(x: Elem, y: Elem, z: Elem) -> Self;
    fn get_x(&self) -> Elem;
    fn get_y(&self) -> Elem;
    fn get_z(&self) -> Elem;

    fn op_dot(self, other: Self) -> Elem {
        self.get_x() * other.get_x() +
            self.get_y() * other.get_y() +
            self.get_z() * other.get_z()
    }

    fn op_magnitude(&self) -> Elem {
        self.op_dot(*self).op_sqrt()
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

    fn op_apply_binary<F: Fn(Elem, Elem) -> Elem>(self, f: F, other: Self) -> Self {
        Self::create(
            f(self.get_x(), other.get_x()),
            f(self.get_y(), other.get_y()),
            f(self.get_z(), other.get_z())
        )
    }

    fn op_minus(self, other: Self) -> Self {
        self.op_apply_binary(|a, b| a - b, other)
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

pub type MultiVec3<N> = cgmath::Vector3<<N as MultiNum>::Scalar>;
pub type MultiUnitVec3<N> = math::UnitVector3<<N as MultiNum>::Scalar>;

pub trait MultiNum {
    type Scalar: Copy + Vec3OpsElem;
    type Bool: Copy;

    const SIZE: u32;

    fn new_vec3_from_iter<It: Iterator<Item = Vec3>>(it: &mut It, default_num: f32) -> MultiVec3<Self>;
    fn new_vec3_array_from_iter<It: Iterator<Item = Vec3>>(
        mut it: It, default_num: f32
    ) -> Vec<MultiVec3<Self>> {
        let mut result = Vec::new();

        let mut peekable = it.peekable();
        while peekable.next().is_some() {
            result.push(Self::new_vec3_from_iter(&mut peekable, default_num));
        }
        result
    }

    fn scalar_zero() -> Self::Scalar;
    fn scalar_one() -> Self::Scalar;
    fn scalar_inf() -> Self::Scalar;
    fn scalar_epsilon() -> Self::Scalar;
    fn scalar_big_epsilon() -> Self::Scalar;

    fn scalar_apprx_eq(a: Self::Scalar, b: Self::Scalar, epsilon: Self::Scalar) -> Self::Bool;
    fn scalar_cmp<Cmp: CmpFn>(a: Self::Scalar, b: Self::Scalar) -> Self::Bool;
    fn scalar_conditional_set(
        cond: Self::Bool, value_true: Self::Scalar, value_false: Self::Scalar
    ) -> Self::Scalar;
    fn scalar_argmin(a: Self::Scalar) -> usize;
    fn get_scalar_arg(a: Self::Scalar, idx: usize) -> f32;

    fn all_true(value: Self::Bool) -> bool;

    fn create_bool_repeating(value: bool) -> Self::Bool;
    fn bool_or(a: Self::Bool, b: Self::Bool) -> Self::Bool;
    fn bool_not(a: Self::Bool) -> Self::Bool;
}

pub struct MultiNum1;
impl MultiNum for MultiNum1 {
    type Scalar = f32;
    type Bool = bool;

    const SIZE: u32 = 1;

    fn new_vec3_from_iter<It: Iterator<Item = Vec3>>(it: &mut It, default_num: f32) -> Vec3 {
        match it.next() {
            Some(value) => value,
            None => Vec3::new(default_num, default_num, default_num)
        }
    }

    fn scalar_zero() -> f32 { 0.0f32 }
    fn scalar_one() -> f32 { 1.0f32 }
    fn scalar_inf() -> f32 { f32::INFINITY }
    fn scalar_epsilon() -> f32 { f32::EPSILON }
    fn scalar_big_epsilon() -> f32 { 0.00003 }

    fn scalar_apprx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        apprx_eq(a, b, epsilon)
    }

    fn scalar_cmp<Cmp: CmpFn>(a: Self::Scalar, b: Self::Scalar) -> Self::Bool {
        Cmp::apply_f32(a, b)
    }

    fn scalar_conditional_set(
        cond: Self::Bool, value_true: Self::Scalar, value_false: Self::Scalar
    ) -> Self::Scalar {
        if cond {
            value_true
        } else {
            value_false
        }
    }

    fn scalar_argmin(a: Self::Scalar) -> usize {
        0
    }

    fn get_scalar_arg(a: Self::Scalar, idx: usize) -> f32 {
        a
    }

    fn all_true(value: Self::Bool) -> bool {
        value
    }

    fn create_bool_repeating(value: bool) -> Self::Bool {
        value
    }

    fn bool_or(a: Self::Bool, b: Self::Bool) -> Self::Bool {
        a || b
    }

    fn bool_not(a: Self::Bool) -> Self::Bool {
        !a
    }
}

#[cfg(target_feature = "sse2")]
pub struct MultiNum4;
#[cfg(target_feature = "sse2")]
impl MultiNum for MultiNum4 {
    type Scalar = SimdFloat4;
    type Bool = SimdFloat4;

    const SIZE: u32 = 4;

    fn new_vec3_from_iter<It: Iterator<Item = Vec3>>(
        it: &mut It, default_num: f32
    ) -> MultiVec3<Self> {
        let mut res = [Align16([default_num; 4]); 3];
        it.take(4).enumerate().for_each(|(i, elem)| {
            res[0].0[i] = elem.x;
            res[1].0[i] = elem.x;
            res[2].0[i] = elem.x;
        });

        MultiVec3::<MultiNum4>::new(
            SimdFloat4::new_load(&res[0]),
            SimdFloat4::new_load(&res[1]),
            SimdFloat4::new_load(&res[2])
        )
    }

    fn scalar_zero() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_zero()) }
    fn scalar_one() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_one()) }
    fn scalar_inf() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_inf()) }
    fn scalar_epsilon() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_epsilon()) }
    fn scalar_big_epsilon() -> SimdFloat4 { SimdFloat4::new_repeating(MultiNum1::scalar_big_epsilon()) }

    fn scalar_apprx_eq(a: SimdFloat4, b: SimdFloat4, epsilon: SimdFloat4) -> Self::Bool {
        a.apprx_eq(b, epsilon)
    }

    fn scalar_cmp<Cmp: CmpFn>(a: Self::Scalar, b: Self::Scalar) -> Self::Bool {
        a.cmp::<Cmp>(b)
    }

    fn scalar_conditional_set(
        cond: Self::Bool, value_true: Self::Scalar, value_false: Self::Scalar
    ) -> Self::Scalar {
        cond.conditional_set(value_true, value_false)
    }

    fn scalar_argmin(a: Self::Scalar) -> usize {
        let arr = a.to_array();
        let mut min_idx_value = (0, arr[0]);
        for idx in 1..4 {
            min_idx_value = (idx, arr[idx])
        }
        min_idx_value.0
    }

    fn get_scalar_arg(a: Self::Scalar, idx: usize) -> f32 {
        a.to_array()[idx]
    }

    fn all_true(value: Self::Bool) -> bool {
        value.test_all_true()
    }

    fn create_bool_repeating(value: bool) -> Self::Bool {
        SimdFloat4::new_bool_repeating(value)
    }

    fn bool_or(a: Self::Bool, b: Self::Bool) -> Self::Bool {
        a.bitwise_or(b)
    }

    fn bool_not(a: Self::Bool) -> Self::Bool {
        !a
    }
}

#[cfg(target_feature = "avx")]
pub struct MultiNum8;
#[cfg(target_feature = "avx")]
impl MultiNum for MultiNum8 {
    type Scalar = SimdFloat8;
    type Bool = SimdFloat8;

    const SIZE: u32 = 8;

    fn new_vec3_from_iter<It: Iterator<Item = Vec3>>(
        it: &mut It, default_num: f32
    ) -> MultiVec3<Self> {
        let mut res = [Align32([default_num; 8]); 3];
        it.take(8).enumerate().for_each(|(i, elem)| {
            res[0].0[i] = elem.x;
            res[1].0[i] = elem.x;
            res[2].0[i] = elem.x;
        });

        MultiVec3::<MultiNum8>::new(
            SimdFloat8::new_load(&res[0]),
            SimdFloat8::new_load(&res[1]),
            SimdFloat8::new_load(&res[2])
        )
    }

    fn scalar_zero() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_zero()) }
    fn scalar_one() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_one()) }
    fn scalar_inf() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_inf()) }
    fn scalar_epsilon() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_epsilon()) }
    fn scalar_big_epsilon() -> SimdFloat8 { SimdFloat8::new_repeating(MultiNum1::scalar_big_epsilon()) }

    fn scalar_apprx_eq(a: Self::Scalar, b: Self::Scalar, epsilon: Self::Scalar) -> Self::Bool {
        a.apprx_eq(b, epsilon)
    }

    fn scalar_cmp<Cmp: CmpFn>(a: Self::Scalar, b: Self::Scalar) -> Self::Bool {
        a.cmp::<Cmp>(b)
    }

    fn scalar_conditional_set(
        cond: Self::Bool, value_true: Self::Scalar, value_false: Self::Scalar
    ) -> Self::Scalar {
        cond.conditional_set(value_true, value_false)
    }

    fn scalar_argmin(a: Self::Scalar) -> usize {
        let arr = a.to_array();
        let mut min_idx_value = (0, arr[0]);
        for idx in 1..8 {
            min_idx_value = (idx, arr[idx])
        }
        min_idx_value.0
    }

    fn get_scalar_arg(a: Self::Scalar, idx: usize) -> f32 {
        a.to_array()[idx]
    }

    fn all_true(value: Self::Bool) -> bool {
        value.test_all_true()
    }

    fn create_bool_repeating(value: bool) -> Self::Bool {
        SimdFloat8::new_bool_repeating(value)
    }

    fn bool_or(a: Self::Bool, b: Self::Bool) -> Self::Bool {
        a.bitwise_or(b)
    }

    fn bool_not(a: Self::Bool) -> Self::Bool {
        !a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use utilities::math::{Vec3, apprx_eq_vec3};

    #[test]
    fn test_cross() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);

        let test_c = a.op_cross(&b);
        let expected_c = Vec3::new(0.0, 0.0, 1.0);

        assert!(apprx_eq_vec3(test_c, expected_c, f32::EPSILON));
    }
}