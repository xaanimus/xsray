extern crate stdsimd;

use std::f32;

use utilities::math::*;

use self::stdsimd::vendor;
use self::stdsimd::simd::f32x8;

pub trait MakesAABoundingBox {
    fn make_aa_bounding_box(&self) -> AABoundingBox;
}

///HasAABB for objects that have axis-aligned bounding boxes
pub trait HasAABoundingBox {
    fn aa_bounding_box_ref(&self) -> &AABoundingBox;

    fn get_bounding_box_center(&self) -> Vec3 {
        let bb = self.aa_bounding_box_ref();
        (bb.lower + bb.upper) / 2.0
    }

    #[cfg(not(target_feature = "avx"))]
    fn intersects_with_bounding_box(&self, _ray: &AABBIntersectionRay) -> bool {
        //TODO retest
        let bb = self.aa_bounding_box_ref();
        let bb_lower: &[f32; 3] = bb.lower.as_ref();
        let bb_upper: &[f32; 3] = bb.upper.as_ref();
        let ray_pos: &[f32; 3] = ray.position.as_ref();
        let inv_dir: &[f32; 3] = inverse_direction.as_ref();

        let (mut t_near_max, mut t_far_min) = (-f32::INFINITY, f32::INFINITY);
        for dimension in 0..3 {
            let t1 = (bb_lower[dimension] - ray_pos[dimension]) * inv_dir[dimension];
            let t2 = (bb_upper[dimension] - ray_pos[dimension]) * inv_dir[dimension];

            let t_near = t1.min(t2);
            let t_far = t1.max(t2);

            t_near_max = t_near_max.max(t_near);
            t_far_min = t_far_min.min(t_far);
        }

        if !(ray.t_range.start <= t_far_min) ||
            !(t_near_max <= ray.t_range.end) ||
            !(t_near_max <= t_far_min)
        {
            return false
        }

        true
    }

    //#[target_feature(enable = "avx")]
    #[cfg(target_feature = "avx")]
    //#[target_feature = "+avx"]
    fn intersects_with_bounding_box(&self, ray: &AABBIntersectionRay) -> bool {
        //TODO might want to look into when an element of inverse_direction = NaN
        let bb = self.aa_bounding_box_ref();
        let bb_vec = unsafe { bb.vec_f32x8() };

        let ray_pos_vec = ray.position;
        let direction_inv_vec = ray.direction_inverse;

        let t_values = (bb_vec - ray_pos_vec) * direction_inv_vec;

        let (t_0, t_1) = unsafe {
            let t_values_low = vendor::_mm256_extractf128_ps(t_values, 0);
            let t_values_high = vendor::_mm256_extractf128_ps(t_values, 1);
            let b0 = vendor::_mm_shuffle_ps(t_values_low, t_values_high, 0b_00_01_00_11);
            let b1 = vendor::_mm_shuffle_ps(b0, b0, 0b_00_10_11_00);
            (t_values_low, b1)
        };

        let t_near = unsafe { vendor::_mm_min_ps(t_0, t_1) };
        let t_far = unsafe { vendor::_mm_max_ps(t_0, t_1) };

        let t_near_max = unsafe {
            let x = t_near;
            let x1 = vendor::_mm_shuffle_ps(x, x, 0b_00_00_00_01);
            let x2 = vendor::_mm_shuffle_ps(x, x, 0b_00_00_00_10);
            let xm = vendor::_mm_max_ss( vendor::_mm_max_ss(x, x1), x2 );
            xm.extract(0)
        };

        let t_far_min = unsafe {
            let x = t_far;
            let x1 = vendor::_mm_shuffle_ps(x, x, 0b_00_00_00_01);
            let x2 = vendor::_mm_shuffle_ps(x, x, 0b_00_00_00_10);
            let xm = vendor::_mm_min_ss( vendor::_mm_min_ss(x, x1), x2 );
            xm.extract(0)
        };

        if !(ray.t_start <= t_far_min) ||

            !(t_near_max <= ray.t_end) ||
            !(t_near_max <= t_far_min)
        {
            return false
        }

        true
    }

}

#[cfg(target_feature = "avx")]
pub struct AABBIntersectionRay {
    pub position: f32x8,
    pub direction_inverse: f32x8,
    pub t_start: f32,
    pub t_end: f32
}

#[cfg(not(target_feature = "avx"))]
pub struct AABBIntersectionRay {
    pub position: Vec3,
    pub direction_inverse: Vec3,
    pub t_start: f32,
    pub t_end: f32
}

impl AABBIntersectionRay {
    #[cfg(target_feature = "avx")]
    pub fn new(ray: &RayUnit) -> AABBIntersectionRay {
        let inverse_direction = Vec3::new(1.0, 1.0, 1.0).div_element_wise(*ray.direction.value());
        AABBIntersectionRay {
            position: f32x8::new(
                ray.position.x, ray.position.y, ray.position.z,
                ray.position.x, ray.position.y, ray.position.z,
                0.0, 0.0),
            direction_inverse: f32x8::new(
                inverse_direction.x, inverse_direction.y, inverse_direction.z,
                inverse_direction.x, inverse_direction.y, inverse_direction.z,
                0.0, 0.0),
            t_start: ray.t_range.start,
            t_end: ray.t_range.end
        }
    }

    #[cfg(not(target_feature = "avx"))]
    pub fn new(ray: &RayUnit) -> AABBIntersectionRay {
        let inverse_direction = Vec3::new(1.0, 1.0, 1.0).div_element_wise(*ray.direction.value());
        AABBIntersectionRay {
            position: ray.position,
            direction_inverse: inverse_direction,
            t_start: ray.t_range.start,
            t_end: ray.t_range.end
        }
    }
}

#[derive(Clone, Debug)]
pub struct AABoundingBox {
    pub lower: Vec3,
    pub upper: Vec3
}

impl AABoundingBox {
    pub fn new() -> AABoundingBox {
        AABoundingBox {
            lower: Vec3::zero(),
            upper: Vec3::zero()
        }
    }

    pub fn empty() -> AABoundingBox {
        AABoundingBox {
            lower: Vec3::new(f32::INFINITY,
                             f32::INFINITY,
                             f32::INFINITY),
            upper: Vec3::new(-f32::INFINITY,
                             -f32::INFINITY,
                             -f32::INFINITY)
        }
    }

    #[cfg(target_feature = "avx")]
    ///This will load the 2 vectors in this bounding box,
    ///and 2 * 4 extra bytes after the bounding box. Make
    ///sure that accessing those extra bytes is legal before
    ///calling this function
    unsafe fn vec_f32x8(&self) -> f32x8 {
        let self_ref: &[f32; 3] = self.lower.as_ref();
        f32x8::load_unchecked(&self_ref[..], 0)
    }
}

impl HasAABoundingBox for AABoundingBox {
    fn aa_bounding_box_ref(&self) -> &AABoundingBox {
        &self
    }
}

pub fn get_aa_bounding_box<T: HasAABoundingBox>(elems: &[T]) -> AABoundingBox {
    let mut full_bounding_box = AABoundingBox::empty();
    for ref elem in elems {
        let bbox: &AABoundingBox = (*elem).aa_bounding_box_ref();
        full_bounding_box.lower = full_bounding_box.lower.min_elem_wise(&bbox.lower);
        full_bounding_box.upper = full_bounding_box.upper.max_elem_wise(&bbox.upper);
    }
    full_bounding_box
}
