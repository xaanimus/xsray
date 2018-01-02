use super::shader::*;
use super::bvh_accelerator::*;
use utilities::math::*;
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


#[derive(Debug)]
pub struct Triangle {
    pub positions: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub shader: Rc<Shader>
}

impl HasSurfaceArea for Triangle {
    fn surface_area(&self) -> f32 {
        (self.positions[1] - self.positions[0]).cross(self.positions[2] - self.positions[0])
            .magnitude() / 2.0
    }
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

pub struct TriangleWithAABoundingBox {
    pub triangle: Triangle,
    aa_bounding_box: AABoundingBox,
    surface_area: f32
}

impl TriangleWithAABoundingBox {
    pub fn new_from_triangle(triangle: &Triangle) -> TriangleWithAABoundingBox {
        TriangleWithAABoundingBox {
            triangle: triangle.clone(),
            aa_bounding_box: triangle.make_aa_bounding_box(),
            surface_area: triangle.surface_area()
        }
    }
}

impl HasAABoundingBox for TriangleWithAABoundingBox {
    fn aa_bounding_box_ref(&self) -> &AABoundingBox {
        &self.aa_bounding_box
    }
}

impl HasSurfaceArea for TriangleWithAABoundingBox {
    fn surface_area(&self) -> f32 {
        self.surface_area
    }
}

#[derive(Debug)]
pub struct IntersectableTriangle {
    triangle: Rc<Triangle>,
    position_0: Vec3,
    edge1: Vec3,
    edge2: Vec3,
}

#[test]
fn test_size() {
    use std::mem::{size_of, align_of};
    println!("IntersectableTriangle size{}, align{}", size_of::<IntersectableTriangle>(), align_of::<IntersectableTriangle>());
    println!("RayUnit size{}, align{}", size_of::<RayUnit>(), align_of::<RayUnit>());
}

impl IntersectableTriangle {
    pub fn new_from_triangle(triangle: &Triangle) -> IntersectableTriangle {
        let triangle_ptr = Rc::new(triangle.clone());
        let edge1 = triangle.positions[1] - triangle.positions[0];
        let edge2 = triangle.positions[2] - triangle.positions[0];
        IntersectableTriangle {
            triangle: triangle_ptr,
            position_0: triangle.positions[0],
            edge1: edge1,
            edge2: edge2,
        }
    }
}

impl Intersectable for IntersectableTriangle {
    //Moller-Trumbore intersection
    //https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    #[target_feature = "+avx"]
    fn intersect_obstruct(&self, ray: &RayUnit, record: &mut IntersectionRecord, _: bool) -> bool {
        let edge1 = self.edge1; //code is not loading these in vectorized mode. align edge
        let edge2 = self.edge2;

        //TODO impl simd cross product
        let h = ray.direction.value().cross(edge2);
        let a = edge1.dot(h);
        if apprx_eq(a, 0.0, f32::EPSILON) {
            return false;
        }

        let f = 1.0 / a;
        let s = ray.position - self.position_0;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return false;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction.value().dot(q);
        if v < 0.0 || u + v > 1.0 {
            return false;
        }

        let t = f * edge2.dot(q);
        if t <= ray.t_range.start || ray.t_range.end <= t || t >= record.t {
            return false;
        }
        if !(t < record.t) {
            return false;
        }

        let beta = u;
        let gamma = v;
        let alpha = 1.0 - beta - gamma;
        *record = IntersectionRecord {
            position: ray.position + t * ray.direction.value(),
            normal: self.triangle.normals[0] * alpha +
                self.triangle.normals[1] * beta +
                self.triangle.normals[2] * gamma,
            t: t,
            shader: Some(self.triangle.shader.clone())
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

