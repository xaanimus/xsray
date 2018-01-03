
extern crate time;
extern crate stdsimd;

use self::stdsimd::vendor;
use self::stdsimd::simd::f32x8;

use std::f32;
use std::cmp::Ordering;

use utilities::math::*;

// Surface Area ==================

pub trait HasSurfaceArea {
    fn surface_area(&self) -> f32;
}

// Bounding Box ==================

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

    //might not work when ray origin is inside box.
    //fn intersects_with_bounding_box(&self, ray: &RayUnit, inverse_direction: &Vec3) -> bool {
    //    let bb = self.aa_bounding_box_ref();
    //    let tvec_lower_bound = (bb.lower - ray.position).mul_element_wise(*inverse_direction);
    //    let tvec_upper_bound = (bb.upper - ray.position).mul_element_wise(*inverse_direction);

    //    //contains minimum t value for x y and z planes
    //    let t_min_xyz = tvec_lower_bound.min_elem_wise(&tvec_upper_bound);
    //    //contains maximum t value for x y and z planes
    //    let t_max_xyz = tvec_lower_bound.max_elem_wise(&tvec_upper_bound);

    //    let t_maximum_of_lower_bounds = t_min_xyz.max();
    //    let t_minimum_of_upper_bounds = t_max_xyz.min();

    //    t_maximum_of_lower_bounds <= t_minimum_of_upper_bounds &&
    //        //bounds check
    //        ray.t_range.start <= t_minimum_of_upper_bounds &&
    //        t_maximum_of_lower_bounds <= ray.t_range.end
    //}

    #[cfg(not(target_feature = "avx"))]
    fn intersects_with_bounding_box(&self, _ray: &AABBIntersectionRay) -> bool {
        unimplemented!()
        ////TODO might want to look into when an element of inverse_direction = NaN
        //let bb = self.aa_bounding_box_ref();
        //let bb_lower: &[f32; 3] = bb.lower.as_ref();
        //let bb_upper: &[f32; 3] = bb.upper.as_ref();
        //let ray_pos: &[f32; 3] = ray.position.as_ref();
        //let inv_dir: &[f32; 3] = inverse_direction.as_ref();

        //let (mut t_near_max, mut t_far_min) = (-f32::INFINITY, f32::INFINITY);
        //for dimension in 0..3 {
        //    let t1 = (bb_lower[dimension] - ray_pos[dimension]) * inv_dir[dimension];
        //    let t2 = (bb_upper[dimension] - ray_pos[dimension]) * inv_dir[dimension];

        //    let t_near = t1.min(t2);
        //    let t_far = t1.max(t2);

        //    t_near_max = t_near_max.max(t_near);
        //    t_far_min = t_far_min.min(t_far);
        //}

        //if !(ray.t_range.start <= t_far_min) ||
        //    !(t_near_max <= ray.t_range.end) ||
        //    !(t_near_max <= t_far_min)
        //{
        //    return false
        //}

        //true
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

#[test]
#[cfg(target_feature = "avx")]
fn test_simd() {
    println!("avx test");
    let t_values = f32x8::new(10.0, 9.0, 8.0, 4.0, 0.0, 0.0, 0.0, 0.0);

    let (a, b) = unsafe {
        let x = vendor::_mm256_extractf128_ps(t_values, 0);
        let x1 = vendor::_mm_shuffle_ps(x, x, 0b_00_00_00_01);
        let x2 = vendor::_mm_shuffle_ps(x, x, 0b_00_00_00_10);
        let xm = vendor::_mm_min_ss( vendor::_mm_min_ss(x, x1), x2 );
        (xm, xm)
    };

    let mut arr = [0.0f32; 4];
    a.store(&mut arr[..], 0);
    println!("{:?}", arr);
}

#[cfg(target_feature = "avx")]
struct AABBIntersectionRay {
    pub position: f32x8,
    pub direction_inverse: f32x8,
    pub t_start: f32,
    pub t_end: f32
}

#[cfg(not(target_feature = "avx"))]
struct AABBIntersectionRay {
    pub position: Vec3,
    pub direction_inverse: Vec3,
    pub t_start: f32,
    pub t_end: f32
}

impl AABBIntersectionRay {
    #[cfg(target_feature = "avx")]
    fn new(ray: &RayUnit) -> AABBIntersectionRay {
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
    fn new(ray: &RayUnit) -> AABBIntersectionRay {
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

macro_rules! print_mem {
    ($type:ty, $fn:ident) => {
        {
            use std;
            println!("{} {} = {}", stringify!($fn), stringify!($type), std::mem::$fn::<$type>());
        }
    }
}

#[test]
fn bounding_box_info() {
    print_mem!(AABoundingBox, align_of);
    print_mem!(AABoundingBox, size_of);
    print_mem!(Vec3, size_of);
}

impl AABoundingBox {
    fn new() -> AABoundingBox {
        AABoundingBox {
            lower: Vec3::zero(),
            upper: Vec3::zero()
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

fn get_aa_bounding_box<T: HasAABoundingBox>(elems: &[T]) -> AABoundingBox {
    let mut full_bounding_box = AABoundingBox::new();
    for ref elem in elems {
        let bbox: &AABoundingBox = (*elem).aa_bounding_box_ref();
        full_bounding_box.lower = full_bounding_box.lower.min_elem_wise(&bbox.lower);
        full_bounding_box.upper = full_bounding_box.upper.max_elem_wise(&bbox.upper);
    }
    full_bounding_box
}

// Bvh Spliting ============================

struct SAHConstants {
    cost_traversal: f32,
    cost_triangle_intersection: f32
}

fn compute_surface_area<T: HasSurfaceArea>(objects: &[T]) -> f32 {
    objects.iter()
        .map(|obj| obj.surface_area())
        .sum()
}

fn surface_area_heuristic<T: HasSurfaceArea>(
    left_objects: &[T],
    right_objects: &[T],
    sah_constants: &SAHConstants
) -> f32 {
    let left_surface_area = compute_surface_area(left_objects);
    let right_surface_area = compute_surface_area(right_objects);
    let total_surface_area = left_surface_area + right_surface_area;
    sah_constants.cost_traversal + sah_constants.cost_triangle_intersection * {
        left_surface_area / total_surface_area * left_objects.len() as f32 +
            right_surface_area / total_surface_area * right_objects.len() as f32
    }
}

trait BVHSplitter {
    /// Computes an index where the bvh should be split.
    /// returns 0 if there should be no split
    fn get_spliting_index<T>(&self, sorted_objects: &[T]) -> usize
        where T: HasAABoundingBox + HasSurfaceArea;
}

struct MedianIndexSplitter;
impl BVHSplitter for MedianIndexSplitter {
    fn get_spliting_index<T>(&self, sorted_objects: &[T]) -> usize
        where T: HasAABoundingBox + HasSurfaceArea
    {
        sorted_objects.len() / 2
    }
}

struct SAHSubdivideGuessSplitter {
    number_of_subdivs: u32,
    sah_consts: SAHConstants
}
impl BVHSplitter for SAHSubdivideGuessSplitter {
    fn get_spliting_index<T: HasAABoundingBox>(&self, sorted_objects: &[T]) -> usize
        where T: HasAABoundingBox + HasSurfaceArea
    {
        if sorted_objects.len() <= 1 {return 0;}

        //the last subdivision may or may not have this size
        let subdivision_size = (sorted_objects.len() as u32 / self.number_of_subdivs).max(1);
        let mut left_size = subdivision_size;
        let mut best_mid_point = 0u32;
        let mut best_cost =
            sorted_objects.len() as f32 * self.sah_consts.cost_triangle_intersection;
        while left_size < sorted_objects.len() as u32 {
            let (left_objects, right_objects) = sorted_objects.split_at(left_size as usize);
            let cost = surface_area_heuristic(left_objects, right_objects, &self.sah_consts);
            if cost < best_cost {
                best_cost = cost;
                best_mid_point = left_size;
            }
            left_size += subdivision_size;
        }

        best_mid_point as usize
    }
}

// BVH =================================

#[derive(Debug)]
pub enum BVHAccelerator {
    Node{first: Box<BVHAccelerator>, second:Box<BVHAccelerator>, wrapper:AABoundingBox},
    Leaf{start: usize, end: usize, wrapper:AABoundingBox},
    Nothing
}

impl BVHAccelerator {
    pub fn new<T: HasAABoundingBox + HasSurfaceArea>(objects: &mut [T]) -> BVHAccelerator {
        let start_time = time::precise_time_s();
        let accelerator = BVHAccelerator::build_tree(objects, 0);
        let end_time = time::precise_time_s();
        println!("bvh build time: {}s", end_time - start_time);
        accelerator
    }

    ///This will mutate objects, so pass in a clone if you don't
    ///want that to happen
    /// start_index is what index objects[0] is in the largest enclosing objects array
    fn build_tree<T: HasAABoundingBox + HasSurfaceArea> (
        objects: &mut [T], start_index: usize
    ) -> BVHAccelerator {
        let objects_bbox = get_aa_bounding_box(objects);

        if objects.len() == 0 {
            return BVHAccelerator::Nothing
        } else if objects.len() <= 4 {
            return BVHAccelerator::Leaf{
                start: start_index,
                end: start_index + objects.len(),
                wrapper: objects_bbox
            };
        }

        //find widest axis
        let dx = objects_bbox.upper.x - objects_bbox.lower.x;
        let dy = objects_bbox.upper.y - objects_bbox.lower.y;
        let dz = objects_bbox.upper.z - objects_bbox.lower.z;
        let x_is_largest = dx >= dy && dx >= dz;
        let y_is_largest = dy >= dx && dy >= dz;

        //sort objects
        if x_is_largest {
            objects.sort_by(|a: &T, b: &T| {
                if a.get_bounding_box_center().x > b.get_bounding_box_center().x {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        } else if y_is_largest {
            objects.sort_by(|a: &T, b: &T| {
                if a.get_bounding_box_center().y > b.get_bounding_box_center().y {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        } else {
            objects.sort_by(|a: &T, b: &T| {
                if a.get_bounding_box_center().z > b.get_bounding_box_center().z {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        };

        //let splitter = MedianIndexSplitter;
        let splitter = SAHSubdivideGuessSplitter {
            number_of_subdivs: 50,
            sah_consts: SAHConstants {
                cost_traversal: 2.0,
                cost_triangle_intersection: 1.0
            }
        };
        let m = splitter.get_spliting_index(objects);
        if m == 0 {
            BVHAccelerator::Leaf {
                start: start_index,
                end: start_index + objects.len(),
                wrapper: objects_bbox
            }
        } else {
            let (left_objects, right_objects) = objects.split_at_mut(m);
            BVHAccelerator::Node {
                first: Box::new(BVHAccelerator::build_tree(left_objects, start_index)),
                second: Box::new(BVHAccelerator::build_tree(right_objects, start_index + m)),
                wrapper: objects_bbox
            }
        }
    }
}

impl BVHAccelerator {
    /// Intersect with bounded boxes. returns true if there is an intersection, and
    /// appends intersection_indices with indices of intersected boxes
    /// If obstruction_only = true, this will return early if it intersects any
    /// box, not triangle.
    fn intersect_box_intern(
        &self, ray: &AABBIntersectionRay,
        obstruction_only: bool, intersection_indices: &mut Vec<usize>
    ) -> bool {
        use self::BVHAccelerator::{Node, Leaf, Nothing};
        match self {
            &Node{ref first, ref second, ref wrapper} => {
                if wrapper.intersects_with_bounding_box(ray) {
                    let first_intersected =
                        first.intersect_box_intern(&ray, obstruction_only, intersection_indices);
                    if obstruction_only && first_intersected {
                        return true;
                    }
                    let second_intersected =
                        second.intersect_box_intern(&ray, obstruction_only, intersection_indices);
                    first_intersected || second_intersected
                } else {
                    false
                }
            },
            &Leaf{start, end, ref wrapper} => {
                if wrapper.intersects_with_bounding_box(ray) {
                    for i in start..end {
                        intersection_indices.push(i)
                    }
                    true
                } else {
                    false
                }
            }
            &Nothing => false
        }
    }

    /// Intersects with bounding boxes to find indices of objects that
    /// may intersect with the ray. Due to the way th bvh tree is build, the
    /// returned indices will already be sorted, so the caller can iterate through
    /// the objects in a way that benefits from cache locality
    pub fn intersect_boxes(&self, ray: &RayUnit, obstruction_only: bool) -> Vec<usize> {
        let mut indices = Vec::<usize>::new();
        let aabb_ray = AABBIntersectionRay::new(ray);
        self.intersect_box_intern(&aabb_ray, obstruction_only, &mut indices);
        indices
    }
}

