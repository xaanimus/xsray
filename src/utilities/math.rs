
extern crate cgmath;

pub use std::ops::{Neg, Range};
use std::f32;
use std::fmt;
use super::multi_math;

// consts =======================
const BIG_EPSILON_F32 : f32 = 0.00003;

// type aliases =================
pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Matrix4 = cgmath::Matrix4<f32>;
pub type Ray = RayBase<MultiNum1>;
pub type UnitVec3 = UnitVector3<f32>;

pub use self::cgmath::{Zero, One, Array, SquareMatrix,
                       InnerSpace, ElementWise, Matrix, ApproxEq};
use self::cgmath::{Vector3, BaseFloat};
use utilities::multi_math::{MultiNum, MultiNum1, Vec3OpsElem, Vec3Ops};

// misc functions ===============

pub fn clamp_i32(x: i32) -> u32 {
    x.max(0) as u32
}

pub fn apprx_eq(a: f32, b: f32, eps: f32) -> bool {
    let x = (b - a).abs();
    x < eps
}

pub fn apprx_eq_vec3(a: Vec3, b: Vec3, eps: f32) -> bool {
    apprx_eq(a.x, b.x, eps) &&
        apprx_eq(a.y, b.y, eps) &&
        apprx_eq(a.z, b.z, eps)
}

// Traits =======================

pub trait HasUnit<T> {
    fn unit(&self) -> T;
}

pub trait HasElemWiseExtrema {
    fn min_elem_wise(&self, other: &Self) -> Self;
    fn max_elem_wise(&self, other: &Self) -> Self;
}

pub trait AccurateOps {
    fn accurate_subtraction(&self, other: &Self) -> Self;
}

//UnitVec3 ======================

#[derive(Debug, Clone)]
pub struct UnitVector3<T: Vec3OpsElem> {
    val: Vector3<T>
}

impl<T: Vec3OpsElem + Copy> UnitVector3<T> {
    pub fn new(value: &Vector3<T>) -> UnitVector3<T> {
        UnitVector3::<T> {
            val: value.op_normalized()
        }
    }

    pub fn value(&self) -> &Vector3<T> {
        &self.val
    }

    pub fn cross(&self, other: UnitVector3<T>) -> Vector3<T> {
        self.val.op_cross(other.val)
    }

    pub fn clone(&self) -> UnitVector3<T> {
        UnitVector3::<T> {
            val: self.val.clone()
        }
    }
}

impl<T: Vec3OpsElem + Copy> Neg for UnitVector3<T> {
    type Output = UnitVector3<T>;
    fn neg(self) -> UnitVector3<T> {
        UnitVector3::<T> {
            val: self.val.op_neg()
        }
    }
}


//Vec3 Extensions ===============
impl From<UnitVec3> for Vec3 {
    fn from(uvec: UnitVec3) -> Self {
        uvec.val
    }
}

///This converts Vec3 into a unit version of Vec3.
///The converted value's type guarantees that it will have a magnitude of 1
impl<T: Vec3OpsElem + Copy> HasUnit<UnitVector3<T>> for Vector3<T> {
    fn unit(&self) -> UnitVector3<T> {
        UnitVector3::<T>::new(&self.clone()) //TODO check if this works without clone
    }
}

impl HasElemWiseExtrema for Vec3 {
    fn min_elem_wise(&self, other: &Vec3) -> Vec3{
        Vec3::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    fn max_elem_wise(&self, other: &Vec3) -> Vec3{
        Vec3::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }
}

impl AccurateOps for Vec3 {
    fn accurate_subtraction(&self, other: &Self) -> Self {
        Vec3::new(
            (self.x as f64 - other.x as f64) as f32,
            (self.y as f64 - other.y as f64) as f32,
            (self.z as f64 - other.z as f64) as f32
        )
    }
}

//Ray ===========================

pub struct RayBase<N: MultiNum> {
    pub position: N::Vector3,
    pub direction: N::UnitVector3,
    pub t_range: Range<N::Scalar>,
}

impl<N: MultiNum> Clone for RayBase<N> {
    fn clone(&self) -> Self {
        RayBase::<N> {
            position: self.position.clone(),
            direction: self.direction.clone(),
            t_range: self.t_range.clone()
        }
    }
}

impl<N: MultiNum> RayBase<N> {
    pub fn new(position: N::Vector3, direction: N::UnitVector3) -> RayBase<N> {
        RayBase {
            position: position,
            direction: direction,
            t_range: N::scalar_zero()..N::scalar_inf()
        }
    }

    pub fn new_epsilon_offset(position: N::Vector3, direction: N::UnitVector3) -> RayBase<N> {
        let mut ray = RayBase::<N>::new(position, direction);
        ray.t_range.start = N::scalar_big_epsilon();
        ray
    }
}

impl<N> fmt::Debug for RayBase<N>
    where N: MultiNum,
          N::Vector3: fmt::Debug,
          N::UnitVector3: fmt::Debug,
          N::Scalar: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RayBase(position: {:?}, direction: {:?}, t_range: {:?})",
            self.position,
            self.direction,
            self.t_range
        )
    }
}
