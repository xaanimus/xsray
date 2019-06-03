use super::simd;
use self::simd::intrin;

pub trait CmpFn {
    fn apply_f32(a: f32, b: f32) -> bool;
    const CMP_CODE: i32;
}

macro_rules! impl_cmp_fn {
    ($cmp_name:tt, opfn = $opfn:expr, intrin_cmp = $intrin_cmp:expr) => {
        pub struct $cmp_name {}
        impl CmpFn for $cmp_name {
            fn apply_f32(a: f32, b: f32) -> bool {
                $opfn(a, b)
            }

            const CMP_CODE: i32 = $intrin_cmp;
        }
    }
}

//#[cfg(target_arch = "x86_64")]
pub mod cmp {
    use super::*;
    impl_cmp_fn!(
        Lt,
        opfn = |a, b| a < b,
        intrin_cmp = intrin::_CMP_LT_OQ
    );
    impl_cmp_fn!(
        Lte,
        opfn = |a, b| a <= b,
        intrin_cmp = intrin::_CMP_LE_OQ
    );
    impl_cmp_fn!(
        Gt,
        opfn = |a, b| a > b,
        intrin_cmp = intrin::_CMP_GT_OQ
    );
    impl_cmp_fn!(
        Gte,
        opfn = |a, b| a >= b,
        intrin_cmp = intrin::_CMP_GE_OQ
    );
}

//#[derive(Clone, Copy, Debug)]
//pub enum CmpFn {
//    Lt, Gt, Lte, Gte
//}
//
//impl CmpFn {
//    pub fn apply_f32(&self, a: f32, b: f32) -> bool {
//        use self::CmpFn::*;
//        match self {
//            Lt => a < b,
//            Gt => a > b,
//            Lte => a <= b,
//            Gte => a >= b
//        }
//    }
//}
