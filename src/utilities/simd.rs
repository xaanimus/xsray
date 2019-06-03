extern crate core;

// TODO clean up simd code in ::engine. try to minimize arch specific code
// TODO test this entire module

use super::math::{Ray, Vec3};
use super::cmp_util::CmpFn;

use std::mem;
use std::ops::{
    Add, Sub, Mul,
    Range, Neg, Div
};

use utilities::multi_math::Sqrt;

macro_rules! impl_simd_trait {
    {trait_name = $tn:ty, fnname = $fnname:ident, binop4 = $binop4:expr, binop8 = $binop8:expr} => {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        impl $tn for SimdFloat4 {
            type Output = SimdFloat4;
            fn $fnname(self, rhs: SimdFloat4) -> SimdFloat4 {
                $binop4(self, rhs)
            }
        }

        #[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
        impl $tn for SimdFloat8 {
            type Output = SimdFloat8;
            fn $fnname(self, rhs: SimdFloat8) -> SimdFloat8 {
                $binop8(self, rhs)
            }
        }
    }
}

impl_simd_trait! {
    trait_name = Div, fnname = div,
    binop4 = |a: SimdFloat4, b: SimdFloat4| unsafe { intrin::_mm_div_ps(a.0, b.0).into() },
    binop8 = |a: SimdFloat8, b: SimdFloat8| unsafe { intrin::_mm256_div_ps(a.0, b.0).into() }
}

#[cfg(target_arch = "x86_64")]
pub use self::core::arch::x86_64 as intrin;
#[cfg(target_arch = "x86_64")]
pub use self::core::arch::x86_64::{__m128, __m256};

// TODO x86_64 might be automatically set if sse2 is set?
#[derive(Debug, Clone, Copy)]
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
pub struct SimdFloat4(__m128);

#[derive(Debug, Clone, Copy)]
#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
pub struct SimdFloat8(__m256);

#[derive(Debug, Clone, Copy)]
#[repr(align(16))]
pub struct Align16<T>(T);

#[derive(Debug, Clone, Copy)]
#[repr(align(32))]
pub struct Align32<T>(T);

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl SimdFloat4 {
    pub fn new_bool_repeating(value: bool) -> SimdFloat4 {
        SimdFloat4::new_bool(value, value, value, value)
    }

    pub fn new_bool(e0: bool, e1: bool, e2: bool, e3: bool) -> SimdFloat4 {
        fn convert(bool_value : bool) -> f32 {
            let int_value = if bool_value {-1i32} else {0i32};
            unsafe {std::mem::transmute::<i32, f32>(int_value)}
        }

        SimdFloat4::new(
            convert(e0),
            convert(e1),
            convert(e2),
            convert(e3)
        )
    }

    pub fn new_repeating(elem: f32) -> SimdFloat4 {
        SimdFloat4::new(elem, elem, elem, elem)
    }

    pub fn new(e0: f32, e1: f32, e2: f32, e3: f32) -> SimdFloat4 {
        // set_ps expects reverse order
        let simdvec = unsafe {intrin::_mm_set_ps(e3, e2, e1, e0)};
        SimdFloat4(simdvec)
    }

    pub fn store(&self, buffer: &mut Align16<[f32; 4]>) {
        let ptr = buffer.0.as_mut_ptr();
        unsafe { intrin::_mm_store_ps(ptr, self.0) }
    }

    pub fn to_array(&self) -> [f32; 4] {
        let mut result = Align16([0f32; 4]);
        self.store(&mut result);
        result.0
    }

    pub fn e0(&self) -> f32 {
        unsafe { intrin::_mm_cvtss_f32(self.0) }
    }

    pub fn m128(&self) -> __m128 {
        self.0
    }

    pub fn vec3_cross(&self, other: SimdFloat4) -> SimdFloat4 {
        let a = self.0;
        let b = other.0;

        unsafe {
            let v1_0 = intrin::_mm256_set_m128(b, a);
            let v1 = intrin::_mm256_permute_ps(v1_0, 0b_00_00_10_01);
            let v2_0 = intrin::_mm256_set_m128(a, b);
            let v2 = intrin::_mm256_permute_ps(v2_0, 0b_00_01_00_10);

            let v_product = intrin::_mm256_mul_ps(v1, v2);
            let v_first = intrin::_mm256_extractf128_ps(v_product, 0);
            let v_second = intrin::_mm256_extractf128_ps(v_product, 1);

            intrin::_mm_sub_ps(v_first, v_second)
        }.into()
    }

    pub fn vec3_dot(&self, other: SimdFloat4) -> f32 {
        let a = self.0;
        let b = other.0;

        let result_vec: SimdFloat4 = unsafe {
            let product = intrin::_mm_mul_ps(a, b);
            let product_1 = intrin::_mm_shuffle_ps(product, product, 0b00_00_00_01);
            let product_2 = intrin::_mm_shuffle_ps(product, product, 0b00_00_00_10);

            intrin::_mm_add_ss(
                intrin::_mm_add_ss(product, product_1),
                product_2)
        }.into();

        result_vec.e0()
    }

    pub fn abs(&self) -> SimdFloat4 {
        unsafe {
            let abs_mask = intrin::_mm_castsi128_ps(
                intrin::_mm_srli_epi32(
                    intrin::_mm_set1_epi32(-1),
                    1
                ));
            intrin::_mm_and_ps(self.0, abs_mask).into()
        }
    }

    pub fn apprx_eq(&self, other: SimdFloat4, epsilon: SimdFloat4) -> SimdFloat4 {
        let diff = (*self - other).abs();
        unsafe {
            intrin::_mm_cmp_ps(
                diff.0,
                epsilon.0,
                intrin::_CMP_LT_OQ
            ).into()
        }
    }

    pub fn test_all_true(&self) -> bool {
        unsafe {
            let all_ones = SimdFloat4::new_bool_repeating(true);
            intrin::_mm_testc_ps(self.0, all_ones.0) == 1
        }
    }

    pub fn cmp<Cmp: CmpFn>(&self, other: SimdFloat4) -> SimdFloat4 {
        unsafe { intrin::_mm_cmp_ps(self.0, other.0, Cmp::CMP_CODE).into() }
    }

    pub fn bitwise_or(&self, other: SimdFloat4) -> SimdFloat4 {
        unsafe { intrin::_mm_or_ps(self.0, other.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl SimdFloat8 {
    pub fn new_bool_repeating(value: bool) -> SimdFloat8 {
        SimdFloat8::new_bool(value, value, value, value, value, value, value, value)
    }

    pub fn new_bool(
        e0: bool, e1: bool, e2: bool, e3: bool,
        e4: bool, e5: bool, e6: bool, e7: bool
    ) -> SimdFloat8 {
        fn convert(bool_value : bool) -> f32 {
            let int_value = if bool_value {-1i32} else {0i32};
            unsafe {std::mem::transmute::<i32, f32>(int_value)}
        }

        SimdFloat8::new(
            convert(e0),
            convert(e1),
            convert(e2),
            convert(e3),
            convert(e4),
            convert(e5),
            convert(e6),
            convert(e7)
        )
    }

    pub fn new_repeating(elem: f32) -> SimdFloat8 {
        SimdFloat8::new(elem, elem, elem, elem, elem, elem, elem, elem)
    }

    pub fn new(e0: f32, e1: f32, e2: f32, e3: f32, e4: f32, e5: f32, e6: f32, e7: f32) -> SimdFloat8 {
        // set_ps expects reverse order
        let simdvec = unsafe { intrin::_mm256_set_ps(e7, e6, e5, e4, e3, e2, e1, e0) };
        SimdFloat8(simdvec)
    }

    pub fn m256(&self) -> __m256 {
        self.0
    }

    pub fn e0(&self) -> f32 {
        unsafe { intrin::_mm256_cvtss_f32(self.0) }
    }

    pub fn store(&self, buffer: &mut Align32<[f32; 8]>) {
        let ptr = buffer.0.as_mut_ptr();
        unsafe { intrin::_mm256_store_ps(ptr, self.0) }
    }

    pub fn to_array(&self) -> [f32; 8] {
        let mut result = Align32([0f32; 8]);
        self.store(&mut result);
        result.0
    }

    pub fn abs(&self) -> SimdFloat8 {
        unsafe {
            let abs_mask = intrin::_mm256_castsi256_ps(
                intrin::_mm256_srli_epi32(
                    intrin::_mm256_set1_epi32(-1),
                    1
                ));
            intrin::_mm256_and_ps(self.0, abs_mask).into()
        }
    }

    pub fn apprx_eq(&self, other: SimdFloat8, epsilon: SimdFloat8) -> SimdFloat8 {
        let diff = (*self - other).abs();
        unsafe {
            intrin::_mm256_cmp_ps(
                diff.0,
                epsilon.0,
                intrin::_CMP_LT_OQ
            ).into()
        }
    }

    pub fn test_all_true(&self) -> bool {
        unsafe {
            let all_ones = SimdFloat8::new_bool_repeating(true);
            intrin::_mm256_testc_ps(self.0, all_ones.0) == 1
        }
    }

    pub fn cmp<Cmp: CmpFn>(&self, other: SimdFloat8) -> SimdFloat8 {
        unsafe { intrin::_mm256_cmp_ps(self.0, other.0, Cmp::CMP_CODE).into() }
    }

    pub fn bitwise_or(&self, other: SimdFloat8) -> SimdFloat8 {
        unsafe { intrin::_mm256_or_ps(self.0, other.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Add for SimdFloat4 {
    type Output = SimdFloat4;

    fn add(self, rhs: SimdFloat4) -> SimdFloat4 {
        unsafe { intrin::_mm_add_ps(self.0, rhs.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Add for SimdFloat8 {
    type Output = SimdFloat8;

    fn add(self, rhs: SimdFloat8) -> SimdFloat8 {
        unsafe { intrin::_mm256_add_ps(self.0, rhs.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Sub for SimdFloat4 {
    type Output = SimdFloat4;

    fn sub(self, rhs: SimdFloat4) -> SimdFloat4 {
        unsafe { intrin::_mm_sub_ps(self.0, rhs.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Sub for SimdFloat8 {
    type Output = SimdFloat8;

    fn sub(self, rhs: SimdFloat8) -> SimdFloat8 {
        unsafe { intrin::_mm256_sub_ps(self.0, rhs.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Neg for SimdFloat4 {
    type Output = SimdFloat4;

    fn neg(self) -> Self::Output {
        // TODO can make this faster by flipping sign bits
        self * SimdFloat4::new(-1.0, -1.0, -1.0, -1.0)
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Neg for SimdFloat8 {
    type Output = SimdFloat8;

    fn neg(self) -> Self::Output {
        // TODO can make this faster by flipping sign bits
        self * SimdFloat8::new(-1.0, -1.0, -1.0, -1.0,
                               -1.0, -1.0, -1.0, -1.0)
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Mul for SimdFloat4 {
    type Output = SimdFloat4;

    fn mul(self, rhs: SimdFloat4) -> SimdFloat4 {
        unsafe { intrin::_mm_mul_ps(self.0, rhs.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Mul for SimdFloat8 {
    type Output = SimdFloat8;

    fn mul(self, rhs: SimdFloat8) -> SimdFloat8 {
        unsafe { intrin::_mm256_mul_ps(self.0, rhs.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Sqrt for SimdFloat4 {
    fn op_sqrt(&self) -> Self {
        unsafe { intrin::_mm_sqrt_ps(self.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl Sqrt for SimdFloat8 {
    fn op_sqrt(&self) -> Self {
        unsafe { intrin::_mm256_sqrt_ps(self.0).into() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl From<__m128> for SimdFloat4 {
    fn from(item: __m128) -> Self {
        SimdFloat4(item)
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl From<__m256> for SimdFloat8 {
    fn from(item: __m256) -> Self {
        SimdFloat8(item)
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl From<SimdFloat4> for __m128 {
    fn from(item: SimdFloat4) -> Self {
        item.0
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl From<SimdFloat8> for __m256 {
    fn from(item: SimdFloat8) -> Self {
        item.0
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl From<Vec3> for SimdFloat4 {
    fn from(item: Vec3) -> Self {
        SimdFloat4::new(item.x, item.y, item.z, 0.0)
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl From<SimdFloat4> for Vec3 {
    fn from(item: SimdFloat4) -> Self {
        let arr = item.to_array();
        Vec3::new(arr[0], arr[1], arr[2])
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
pub struct SimdRay {
    pub position: SimdFloat4,
    /// Invariant: direction is always a unit vector
    pub direction: SimdFloat4,
    pub t_range: Range<f32>
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
impl SimdRay {
    pub fn new(ray: &Ray) -> SimdRay {
        SimdRay {
            position: ray.position.into(),
            direction: (*ray.direction.value()).into(),
            t_range: ray.t_range.clone()
        }
    }
}

/// allows evaluating one or another expression depending on whether avx
/// is enabled in the current build
macro_rules! if_avx {
    (avx = $avx_expr:expr, noavx = $no_avx_expr:expr) => {
        {
            #[cfg(target_feature = "avx")]
            let result = {
                $avx_expr
            };

            #[cfg(not(target_feature = "avx"))]
            let result = {
                $no_avx_expr
            };

            result
        }
    }
}

// TESTS

#[cfg(test)]
#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
mod tests {
    use super::*;

    #[test]
    fn test_float4() {

        let test_vec = SimdFloat4::new(1.0, 2.0, 3.0, 4.0);
        let test_arr = test_vec.to_array();
        let expected_arr = [1.0f32, 2.0, 3.0, 4.0];
        assert_array_eq!(test_arr, expected_arr);

        // e0
        assert_eq!(test_vec.e0(), 1.0);

        let a = SimdFloat4::new(1.0, 3.0, 2.0, 0.0);
        let b = SimdFloat4::new(30.0, 2.0, 1.0, 0.0);

        // cross product
        let test_cross = a.vec3_cross(b);
        let test_cross_arr = test_cross.to_array();
        let expected_cross = [-1.0, 59.0, -88.0];
        for i in 0..3 {
            assert_near!(test_cross_arr[i], expected_cross[i], 0.00001);
        }

        //dot product
        let test_dot = a.vec3_dot(b);
        let expected_dot = 38.0f32;
        assert_near!(test_dot, expected_dot, 0.00001);

        // arithmetic
        let test_arithmetic = (a + b - a * b).to_array();
        let expected_arithmetic =
            SimdFloat4::new(1.0, -1.0, 1.0, 0.0).to_array();
        for i in 0..4 {
            assert_near!(test_arithmetic[i], expected_arithmetic[i], 0.0001);
        }
    }

    #[test]
    fn test_bool_ops() {
        assert!(SimdFloat4::new_bool_repeating(true).test_all_true());
        assert!(SimdFloat8::new_bool_repeating(true).test_all_true());

        let false_simd4 = [
            SimdFloat4::new_bool(false, true, true, true),
            SimdFloat4::new_bool(false, false, true, true),
            SimdFloat4::new_bool(false, false, false, false),
            SimdFloat4::new_bool(false, false, false, true)
        ];

        let false_simd8 = [
            SimdFloat8::new_bool_repeating(false),
            SimdFloat8::new_bool(false, false, false, false, false, false, true, false),
            SimdFloat8::new_bool(true, false, false, false, false, false, true, false),
            SimdFloat8::new_bool(true, false, true, false, false, false, true, false),
            SimdFloat8::new_bool(true, false, false, false, false, false, false, false),
        ];

        for elem in &false_simd4 {
            assert!(!elem.test_all_true());
        }
        for elem in &false_simd8 {
            assert!(!elem.test_all_true());
        }
    }

    #[test]
    fn test_float8() {
        let test_vec = SimdFloat8::new(
            100.0, 0.0, 2.0, 3.0,
            4.0, 2.0, 2.0, 3.0);

        assert_eq!(test_vec.e0(), 100.0);

        let test_array = test_vec.to_array();
        let expected_array = [
            100.0f32, 0.0, 2.0, 3.0,
            4.0, 2.0, 2.0, 3.0
        ];
        assert_array_eq!(test_array, expected_array);
    }
}
