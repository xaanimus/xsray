//!Core utilities

pub mod sampler;
pub mod math;
pub mod color;
#[macro_use]
pub mod codable;

macro_rules! print_mem {
    ($type:ty, $fn:ident) => {
        {
            use std;
            println!("{} {} = {}", stringify!($fn), stringify!($type), std::mem::$fn::<$type>());
        }
    }
}

macro_rules! dbg {
    ($fqn:path) => {
        println!("{} = {:?}", stringify!($fqn), $fqn)
    }
}
