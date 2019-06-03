//!Core utilities

#[macro_use]
pub mod test_helpers;

pub mod sampler;
pub mod math;
pub mod multi_math;
pub mod color;
#[macro_use]
pub mod codable;
#[macro_use]
pub mod simd;

macro_rules! print_mem {
    ($type:ty, $fn:ident) => {
        {
            use std;
            println!("{} {} = {}", stringify!($fn), stringify!($type), std::mem::$fn::<$type>());
        }
    }
}

macro_rules! dbg {
    ($x:expr) => {
        println!("{} = {:?}", stringify!($x), $x)
    }
}

