extern crate time;
extern crate stdsimd;

use self::stdsimd::vendor;
use self::stdsimd::simd::f32x8;

use super::aabb::*;
use super::splitter::*;

use std::f32;
use std::cmp::Ordering;
use std::ops::Range;
use std::collections::VecDeque;

use utilities::math::*;

#[derive(Debug)]
pub enum BVHAccelerator {
    Node{first: Box<BVHAccelerator>, second: Box<BVHAccelerator>, wrapper: AABoundingBox},
    Leaf{start: usize, end: usize, wrapper: AABoundingBox},
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
        } else if objects.len() <= 1 {
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
    /// Intersect with bounded boxes.
    /// appends intersection_indices with indices of intersected boxes
    /// TODO make this function iterative. recursion is expensive
    fn intersect_box_intern(
        &self, ray: &AABBIntersectionRay,
        intersection_indices: &mut Vec<Range<usize>>
    ) {
        use self::BVHAccelerator::{Node, Leaf, Nothing};
        let mut nodes_to_visit = VecDeque::<&BVHAccelerator>::new();
        nodes_to_visit.push_front(self);

        while let Some(node) = nodes_to_visit.pop_front() {
            match node {
                &Node{ref first, ref second, ref wrapper}
                if wrapper.intersects_with_bounding_box(ray) => {
                    nodes_to_visit.push_front(&second);
                    nodes_to_visit.push_front(&first);
                },
                &Leaf{start, end, ref wrapper}
                if wrapper.intersects_with_bounding_box(ray) => {
                    intersection_indices.push(start..end);
                },
                _ => ()
            }
        }
    }

    /// Intersects with bounding boxes to find indices of objects that
    /// may intersect with the ray. Due to the way th bvh tree is build, the
    /// returned indices will already be sorted, so the caller can iterate through
    /// the objects in a way that benefits from cache locality
    pub fn intersect_boxes(&self, ray: &RayUnit) -> Vec<Range<usize>> {
        let mut indices = Vec::<Range<usize>>::new();
        let aabb_ray = AABBIntersectionRay::new(ray);
        self.intersect_box_intern(&aabb_ray, &mut indices);
        indices
    }
}
