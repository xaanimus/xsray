use super::math::*;
use std::cmp::Ordering;

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

    fn intersects_with_bounding_box(&self, ray: &RayUnit) -> bool {
        let direction: &Vec3 = ray.direction.vec();

        let (close_x, far_x) = if direction.x > 0.0 {
            (self.aa_bounding_box().lower.x, self.aa_bounding_box().upper.x)
        } else {
            (self.aa_bounding_box().upper.x, self.aa_bounding_box().lower.x)
        };
        let (close_y, far_y) = if direction.y > 0.0 {
            (self.aa_bounding_box().lower.y, self.aa_bounding_box().upper.y)
        } else {
            (self.aa_bounding_box().upper.y, self.aa_bounding_box().lower.y)
        };
        let (close_z, far_z) = if direction.z > 0.0 {
            (self.aa_bounding_box().lower.z, self.aa_bounding_box().upper.z)
        } else {
            (self.aa_bounding_box().upper.z, self.aa_bounding_box().lower.z)
        };

        let (txmin, txmax) = (
            (close_x - ray.position.x) * direction.x,
            (far_x - ray.position.x) * direction.x
        );
        let (tymin, tymax) = (
            (close_y - ray.position.y) * direction.y,
            (far_y - ray.position.y) * direction.y
        );
        let (tzmin, tzmax) = (
            (close_z - ray.position.z) * direction.z,
            (far_z - ray.position.z) * direction.z
        );

        let line_intersects = !(
            txmin > tymax || txmin > tzmax ||
            tymin > txmax || tymin > tzmax ||
            tzmin > txmax || tzmin > tymax
        );

        line_intersects && txmax > 0.0 && tymax > 0.0 && tzmax > 0.0
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
    Leaf(T)
}

impl<T: HasAABoundingBox + Clone> BVHAccelerator<T> {

    pub fn new(objects: &[T]) -> BVHAccelerator<T> {
        BVHAccelerator::build_tree(&mut objects.to_vec())
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
        let x_is_largest = dx > dy && dx > dz;
        let y_is_largest = dy > dx && dy > dz;

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

