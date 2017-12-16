use super::shader::*;
use super::math::*;
use super::bvh_accelerator::*;
use std::rc::Rc;
use std::f32;

pub trait Intersectable {
    /// check for intersection between ray and surface.
    /// returns IntersectionRecord with t=inf if no intersection
    fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        self.intersect_obstruct(ray, false)
    }

    fn intersect_obstruct(&self, ray: &RayUnit, obstruction_only: bool) -> IntersectionRecord;
}

#[derive(Debug, Clone)]
pub struct BVHTriangleWrapper {
    pub triangle: Triangle,
    pub bounding_box: AABoundingBox,
}

impl BVHTriangleWrapper {
    pub fn new(triangle: Triangle) -> BVHTriangleWrapper {
        let bounding_box =
            AABoundingBox {
                lower: triangle.positions[0]
                    .min_elem_wise(&triangle.positions[1])
                    .min_elem_wise(&triangle.positions[2]),
                upper: triangle.positions[0]
                    .max_elem_wise(&triangle.positions[1])
                    .max_elem_wise(&triangle.positions[2])
            };
        BVHTriangleWrapper {
            triangle: triangle,
            bounding_box: bounding_box
        }
    }
}

impl Intersectable for BVHTriangleWrapper {
    fn intersect_obstruct(&self, ray: &RayUnit, obstruction_only: bool) -> IntersectionRecord {
        self.triangle.intersect(ray)
    }
}

impl HasAABoundingBox for BVHTriangleWrapper {
    fn aa_bounding_box(&self) -> &AABoundingBox {
        &self.bounding_box
    }
}

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

impl Intersectable for Triangle {
    fn intersect_obstruct(&self, ray: &RayUnit, _: bool) -> IntersectionRecord {

        //using barymetric coordinates to intersect with this triangle
        // vectors a, b, and c are the 0, 1, and 2 vertices for this triangle
        let a_col_1 = self.positions[0] - self.positions[1]; //a - b
        let a_col_2 = self.positions[0] - self.positions[2]; //a - c
        let a_col_3 = ray.direction.value(); //d
        let b_col = self.positions[0] - ray.position;

        //compute determinant of A
        let mut a_mat = Matrix3::from_cols(a_col_1, a_col_2, *a_col_3);
        let det_a = a_mat.determinant();

        // Checking that the determinant of A is
        // nonzero is unnecessary. If the determinant is
        // 0 (which is rare), the t computed will be NaN,
        // which will make this function correctly return
        // no intersection

        //compute determinant of A_1
        a_mat.x = b_col;
        let det_a1 = a_mat.determinant();

        //compute determinant of A_2
        a_mat.y = b_col;
        a_mat.x = a_col_1;
        let det_a2 = a_mat.determinant();

        //compute determinant of A_3
        a_mat.z = b_col;
        a_mat.y = a_col_2;
        let det_a3 = a_mat.determinant();

        //calculate coordinates
        let beta = det_a1 / det_a;
        let gamma = det_a2 / det_a;
        let t = det_a3 / det_a;

        //test inside traignle and t range
        if beta + gamma < 1.0 && beta > 0.0 && gamma > 0.0 &&
            ray.t_range.start < t && t < ray.t_range.end
            {
                let alpha = 1.0 - beta - gamma;
                //interpolate
                IntersectionRecord {
                    position: ray.position + t * ray.direction.value(),
                    normal: self.normals[0] * alpha + self.normals[1] * beta + self.normals[2] * gamma,
                    t: t,
                    shader: Some(self.shader.clone())
                }
            } else {
            IntersectionRecord::no_intersection()
        }
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

