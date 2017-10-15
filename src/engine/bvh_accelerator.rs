use super::math::*;
use super::intersectable::{IntersectionRecord, Intersectable};
use super::shader::Shader;
use std::cmp::Ordering;
use std::rc::Rc;
use std::f32;

#[derive(Clone, Debug)]
pub struct AABoundingBox {
    pub lower: Vec3,
    pub upper: Vec3
}

impl AABoundingBox {
    fn new() -> AABoundingBox {
        AABoundingBox {
            lower: Vec3::zero(),
            upper: Vec3::zero()
        }
    }
}

impl HasAABoundingBox for AABoundingBox {
    fn aa_bounding_box(&self) -> &AABoundingBox {
        &self
    }
}

///HasAABB for objects that have axis-aligned bounding boxes
pub trait HasAABoundingBox {
    fn aa_bounding_box(&self) -> &AABoundingBox;

    fn get_bounding_box_center(&self) -> Vec3 {
        let bb = self.aa_bounding_box();
        (bb.lower + bb.upper) / 2.0
    }

    fn intersects_with_bounding_box(&self, ray: &RayUnit, inverse_direction: &Vec3) -> bool {
        let bb = self.aa_bounding_box();
        let tvecLowerBound = (bb.lower - ray.position).mul_element_wise(*inverse_direction);
        let tvecUpperBound = (bb.upper - ray.position).mul_element_wise(*inverse_direction);

        let tMinX = tvecLowerBound.x.min(tvecUpperBound.x);
        let tMinY = tvecLowerBound.y.min(tvecUpperBound.y);
        let tMinZ = tvecLowerBound.z.min(tvecUpperBound.z);
        let tMaxX = tvecLowerBound.x.max(tvecUpperBound.x);
        let tMaxY = tvecLowerBound.y.max(tvecUpperBound.y);
        let tMaxZ = tvecLowerBound.z.max(tvecUpperBound.z);

        let t_maximum_of_lower_bounds = tMinX.max(tMinY).max(tMinZ);
        //TODO BUG
        //TODO update this to check that the intersection of the box trange and the ray trange is nonempty
        t_maximum_of_lower_bounds <= tMaxX.min(tMaxY).min(tMaxZ) &&
            ray.t_range.start <= t_maximum_of_lower_bounds &&
            t_maximum_of_lower_bounds <= ray.t_range.end
    }
}

fn get_aa_bounding_box<T: HasAABoundingBox>(elems: &[T]) -> AABoundingBox {
    let mut full_bounding_box = AABoundingBox::new();
    for ref elem in elems {
        let bbox: &AABoundingBox = (*elem).aa_bounding_box();
        full_bounding_box.lower = full_bounding_box.lower.min_elem_wise(&bbox.lower);
        full_bounding_box.upper = full_bounding_box.upper.max_elem_wise(&bbox.upper);
    }
    full_bounding_box
}

#[derive(Debug)]
pub enum BVHAccelerator {
    Node{first: Box<BVHAccelerator>, second:Box<BVHAccelerator>, wrapper:AABoundingBox},
    Leaf{start: usize, end: usize},
    Nothing
}

impl BVHAccelerator {
    pub fn new<T: HasAABoundingBox>(objects: &mut [T]) -> BVHAccelerator {
        BVHAccelerator::build_tree(objects, 0)
    }

    ///This will mutate objects, so pass in a clone if you don't
    ///want that to happen
    /// start_index is what index objects[0] is in the largest enclosing objects array
    fn build_tree<T: HasAABoundingBox>(objects: &mut [T], start_index: usize) -> BVHAccelerator {

        if objects.len() == 0 {
            return BVHAccelerator::Nothing
        } else if objects.len() == 1 {
            return BVHAccelerator::Leaf{start: start_index, end: start_index + 1};
        }

        //find widest axis
        let objects_bbox = get_aa_bounding_box(objects);
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

        let m = objects.len() / 2;
        let (left_objects, right_objects) = objects.split_at_mut(m);
        BVHAccelerator::Node {
            first: Box::new(BVHAccelerator::build_tree(left_objects, start_index)),
            second: Box::new(BVHAccelerator::build_tree(right_objects, start_index + m)),
            wrapper: objects_bbox
        }
    }
}

impl BVHAccelerator {
    /// Intersect with bounded boxes. returns true if there is an intersection, and
    /// appends intersection_indices with indices of intersected boxes
    fn intersect_box_intern(
        &self, ray: &RayUnit, inverse_direction: &Vec3,
        obstruction_only: bool, intersection_indices: &mut Vec<usize>
    ) -> bool {
        use self::BVHAccelerator::{Node, Leaf, Nothing};
        match self {
            &Node{ref first, ref second, ref wrapper} => {
                if wrapper.intersects_with_bounding_box(ray, inverse_direction) {
                    let first_intersected =
                        first.intersect_box_intern(&ray, inverse_direction, obstruction_only, intersection_indices);
                    if obstruction_only && first_intersected {
                        return true;
                    }
                    let second_intersected =
                        second.intersect_box_intern(&ray, inverse_direction, obstruction_only, intersection_indices);
                    first_intersected || second_intersected
                } else {
                    false
                }
            },
            &Leaf{start, end} => {
                for i in start..end {
                    intersection_indices.push(i)
                }
                true
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
        let inverse_direction = Vec3::new(1.0, 1.0, 1.0).div_element_wise(*ray.direction.vec());
        self.intersect_box_intern(ray, &inverse_direction, obstruction_only, &mut indices);
        indices
    }
}

