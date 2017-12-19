use super::shader::*;
use super::math::*;
use super::bvh_accelerator::*;
use std::rc::Rc;
use std::f32;

pub trait Intersectable {
    /// check for intersection between ray and surface.
    /// if there is an intersection, fills record with intersection information
    /// only if the new intersection's t is less than the old intersection's t and return true
    /// if there is no intersection, leave record alone and return false
    fn intersect(&self, ray: &RayUnit, record: &mut IntersectionRecord) -> bool {
        self.intersect_obstruct(ray, record, false)
    }

    fn intersect_obstruct(
        &self, ray: &RayUnit,
        record: &mut IntersectionRecord,
        obstruction_only: bool
    ) -> bool;
}

//#[derive(Debug, Clone)]
//pub struct BVHTriangleWrapper {
//    pub triangle: Triangle,
//    pub bounding_box: AABoundingBox,
//}
//
//impl BVHTriangleWrapper {
//    pub fn new(triangle: Triangle) -> BVHTriangleWrapper {
//        let bounding_box =
//            AABoundingBox {
//                lower: triangle.positions[0]
//                    .min_elem_wise(&triangle.positions[1])
//                    .min_elem_wise(&triangle.positions[2]),
//                upper: triangle.positions[0]
//                    .max_elem_wise(&triangle.positions[1])
//                    .max_elem_wise(&triangle.positions[2])
//            };
//        BVHTriangleWrapper {
//            triangle: triangle,
//            bounding_box: bounding_box
//        }
//    }
//}
//impl Intersectable for BVHTriangleWrapper {
//    fn intersect_obstruct(&self, ray: &RayUnit, obstruction_only: bool) -> IntersectionRecord {
//        self.triangle.intersect(ray)
//    }
//}
//
//impl HasAABoundingBox for BVHTriangleWrapper {
//    fn aa_bounding_box_ref(&self) -> &AABoundingBox {
//        &self.bounding_box
//    }
//}

#[derive(Debug)]
pub struct Triangle {
    pub positions: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub shader: Rc<Shader>
}

impl Clone for Triangle {
    fn clone(&self) -> Triangle {
        Triangle {
            positions: [self.positions[0].clone(),
                self.positions[1].clone(),
                self.positions[2].clone()],
            normals: [self.normals[0].clone(),
                self.normals[1].clone(),
                self.normals[2].clone()],
            shader: self.shader.clone()
        }
    }
}

impl MakesAABoundingBox for Triangle {
    fn make_aa_bounding_box(&self) -> AABoundingBox {
        AABoundingBox {
            lower: self.positions[0]
                .min_elem_wise(&self.positions[1])
                .min_elem_wise(&self.positions[2]),
            upper: self.positions[0]
                .max_elem_wise(&self.positions[1])
                .max_elem_wise(&self.positions[2])
        }
    }
}

#[derive(Debug)]
pub struct IntersectableTriangle {
    triangle: Triangle,
    aa_bounding_box: AABoundingBox,
    a_col_1: Vec3,
    a_col_2: Vec3,
    small_det_12: f32
}

impl IntersectableTriangle {
    pub fn new_from_triangle(triangle: &Triangle) -> IntersectableTriangle {
        let a_col_1 = triangle.positions[0] - triangle.positions[1]; //a - b
        let a_col_2 = triangle.positions[0] - triangle.positions[2]; //a - c
        IntersectableTriangle {
            triangle: triangle.clone(),
            aa_bounding_box: triangle.make_aa_bounding_box(),
            a_col_1: a_col_1,
            a_col_2: a_col_2,
            small_det_12: a_col_1.y * a_col_2.z - a_col_2.y * a_col_1.z
        }
    }
}

impl HasAABoundingBox for IntersectableTriangle {
    fn aa_bounding_box_ref(&self) -> &AABoundingBox {
        &self.aa_bounding_box
    }
}

impl Intersectable for IntersectableTriangle {
    //#[target_feature = "+avx"]
    fn intersect_obstruct(&self, ray: &RayUnit, record: &mut IntersectionRecord, _: bool) -> bool {
        let triangle = &self.triangle;

        //using cramer's rule
        //using barymetric coordinates to intersect with this triangle
        // vectors a, b, and c are the 0, 1, and 2 vertices for this triangle
        let a_col_1 = &self.a_col_1; //a - b
        let a_col_2 = &self.a_col_2; //a - c
        let a_col_3 = ray.direction.value(); //d
        let b_col = triangle.positions[0] - ray.position; //critical, vectorize

        let small_det_23 = a_col_2.y * a_col_3.z - a_col_3.y * a_col_2.z;
        let small_det_13 = a_col_1.y * a_col_3.z - a_col_3.y * a_col_1.z;
        let small_det_12 = self.small_det_12;
        let small_det_1b = a_col_1.y * b_col.z - b_col.y * a_col_1.z;
        let small_det_2b = a_col_2.y * b_col.z - b_col.y * a_col_2.z;

        //compute determinant of A
        let det_a = a_col_1.x * small_det_23 - a_col_2.x * small_det_13 + a_col_3.x * small_det_12;
        // Checking that the determinant of A is
        // nonzero is unnecessary. If the determinant is
        // 0 (which is rare), the t computed will be NaN,
        // which will make this function correctly return
        // no intersection

        //compute determinant of A_3
        let det_a3 = a_col_1.x * small_det_2b - a_col_2.x * small_det_1b + b_col.x * small_det_12;
        let t = det_a3 / det_a;
        if t <= ray.t_range.start || ray.t_range.end <= t || t >= record.t {
            return false;
        }

        //compute determinant of A_1
        let small_det_b2 = b_col.y * a_col_2.z - a_col_2.y * b_col.z;
        let small_det_b3 = b_col.y * a_col_3.z - a_col_3.y * b_col.z;
        let det_a1 = b_col.x * small_det_23 - a_col_2.x * small_det_b3 + a_col_3.x * small_det_b2;
        let beta = det_a1 / det_a;
        if beta <= 0.0 || 1.0 <= beta {
            return false;
        }

        //compute determinant of A_2
        let det_a2 = a_col_1.x * small_det_b3 - b_col.x * small_det_13 + a_col_3.x * small_det_1b;
        let gamma = det_a2 / det_a;
        if gamma <= 0.0 || 1.0 <= gamma + beta {
            return false;
        }

        let alpha = 1.0 - beta - gamma;
        *record = IntersectionRecord {
            position: ray.position + t * ray.direction.value(),
            normal: triangle.normals[0] * alpha +
                triangle.normals[1] * beta +
                triangle.normals[2] * gamma,
            t: t,
            shader: Some(triangle.shader.clone())
        };
        return true;
    }
}

#[derive(Clone)]
pub struct IntersectionRecord {
    pub shader: Option<Rc<Shader>>,
    pub position: Vec3,
    pub normal: Vec3,
    pub t: f32
}

impl IntersectionRecord {
    //TODO make this a static constant?
    pub fn no_intersection() -> IntersectionRecord {
        IntersectionRecord {
            shader: None,
            position: Vec3{x: 0., y: 0., z: 0.},
            normal: Vec3{x: 0., y: 0., z: 0.},
            t: f32::INFINITY
        }
    }

    pub fn intersected(&self) -> bool {
        self.t.is_finite()
    }
}

