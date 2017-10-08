use super::math::*;
use super::{Intersectable, IntersectionRecord, Shader};
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

    //consider precomputing inverse of ray direction
    fn intersects_with_bounding_box(&self, ray: &RayUnit) -> bool {
        let bb = self.aa_bounding_box();
        let tvecLowerBound = (bb.lower - ray.position).div_element_wise(*ray.direction.vec());
        let tvecUpperBound = (bb.upper - ray.position).div_element_wise(*ray.direction.vec());

        let tMinX = tvecLowerBound.x.min(tvecUpperBound.x);
        let tMinY = tvecLowerBound.y.min(tvecUpperBound.y);
        let tMinZ = tvecLowerBound.z.min(tvecUpperBound.z);
        let tMaxX = tvecLowerBound.x.max(tvecUpperBound.x);
        let tMaxY = tvecLowerBound.y.max(tvecUpperBound.y);
        let tMaxZ = tvecLowerBound.z.max(tvecUpperBound.z);

        tMinX.max(tMinY).max(tMinZ) <= tMaxX.min(tMaxY).min(tMaxZ)
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
pub enum BVHAccelerator<T: HasAABoundingBox> {
    Node{first: Box<BVHAccelerator<T>>, second:Box<BVHAccelerator<T>>, wrapper:AABoundingBox},
    Leaf(T),
    Nothing
}

impl<T: HasAABoundingBox + Intersectable + Clone> BVHAccelerator<T> {

    pub fn new(objects: &[T]) -> BVHAccelerator<T> {
        BVHAccelerator::build_tree(&mut objects.to_vec())
    }

    pub fn new_into(objects: Box<[T]>) -> BVHAccelerator<T> {
        BVHAccelerator::build_tree(&mut objects.into_vec())
    }

    ///This will mutate objects, so pass in a clone if you don't
    ///want that to happen
    fn build_tree(objects: &mut [T]) -> BVHAccelerator<T> {
        if objects.len() == 1 {
            return BVHAccelerator::Leaf(objects[0].clone());
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
        let m = objects.len()/2;
        let (left_objects, right_objects) = objects.split_at_mut(m);
        BVHAccelerator::Node{
            first: Box::new(BVHAccelerator::build_tree(left_objects)),
            second: Box::new(BVHAccelerator::build_tree(right_objects)),
            wrapper: objects_bbox
        }
    }
}

//TODO take into account ray range
impl<T: HasAABoundingBox + Intersectable> Intersectable for BVHAccelerator<T> {

    fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        use self::BVHAccelerator::{Node, Leaf};
        match self {
            &Node{ref first, ref second, ref wrapper} => {
                if wrapper.intersects_with_bounding_box(ray) {
                    let (first_intersection, second_intersection) =
                        (first.intersect(&ray), second.intersect(&ray));

                    if first_intersection.t < second_intersection.t {
                        first_intersection
                    } else {
                        second_intersection
                    }

                } else {
                    return IntersectionRecord::no_intersection()
                }
            },
            &Leaf(ref elem) => elem.intersect(ray),
            ref Nothing => IntersectionRecord::no_intersection()
        }
    }
}
