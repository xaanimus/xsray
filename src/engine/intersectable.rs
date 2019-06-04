
extern crate cgmath;

use super::shader::*;
use super::bvh::*;
use super::transformable::*;

use utilities::math::*;

#[cfg(all(target_feature = "avx"))]
use utilities::simd::{
    SimdFloat4,
    SimdRay,
    intrin,
    __m128, __m256
};

use std::rc::Rc;
use std::f32;

use self::cgmath::Transform;
use engine::intersect_util::IntersectionResult;
use utilities::multi_math::MultiNum1;

pub enum IntersectionOrderKind {
    FirstIntersection,
    AnyIntersection
}

pub struct IntersectionArgs<'a> {
//    #[cfg(target_feature = "avx")]
//    pub ray: &'a SimdRay,
//    #[cfg(not(target_feature = "avx"))]
    pub ray: &'a Ray,
    pub record: &'a mut IntersectionRecord,
    pub intersection_order: IntersectionOrderKind
}

pub trait Intersectable {
    /// check for intersection between ray and surface.
    /// if there is an intersection, fills record with intersection information
    /// only if the new intersection's t is less than the old intersection's t and return true
    /// if there is no intersection, leave record alone and return false
    fn intersect(&self, args: IntersectionArgs) -> bool;
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

impl Transformable for Triangle {
    /// Attempts to transform if the transform is invertible.
    /// If the transform is not invertible, normals will be invalid.
    fn transform_in_place(&mut self, transform: &Matrix4) {
        for position in self.positions.iter_mut() {
            *position = (transform * position.extend(1.0)).truncate();
        }

        let normal_transform: Matrix4 = {
            let mut nt = transform.clone();
            nt.w = Vec4::new(0.0, 0.0, 0.0, 1.0);
            nt.invert().unwrap_or(<Matrix4 as One>::one()).transpose()
        };

        for normal in self.normals.iter_mut() {
            *normal = normal_transform.transform_vector(*normal);
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
//#[cfg(not(target_feature = "avx"))]
pub struct IntersectableTriangle {
    triangle: Rc<Triangle>,
    position_0: Vec3,
    edge1: Vec3,
    edge2: Vec3
}

//#[derive(Debug)]
//#[cfg(target_feature = "avx")]
//pub struct IntersectableTriangle {
//    triangle: Rc<Triangle>,
//    position_0: SimdFloat4,
//    edge1: SimdFloat4,
//    edge2: SimdFloat4,
//}

impl IntersectableTriangle {
//    #[cfg(not(target_feature = "avx"))]
    pub fn new_from_triangle(triangle: &Triangle) -> IntersectableTriangle {
        let triangle_ptr = Rc::new(triangle.clone());
        let edge1 = triangle.positions[1].accurate_subtraction(&triangle.positions[0]);
        let edge2 = triangle.positions[2].accurate_subtraction(&triangle.positions[0]);
        IntersectableTriangle {
            triangle: triangle_ptr,
            position_0: triangle.positions[0],
            edge1: edge1,
            edge2: edge2,
        }
    }

//    #[cfg(target_feature = "avx")]
//    pub fn new_from_triangle(triangle: &Triangle) -> IntersectableTriangle {
//        let triangle_ptr = Rc::new(triangle.clone());
//        let edge1 = triangle.positions[1].accurate_subtraction(&triangle.positions[0]);
//        let edge2 = triangle.positions[2].accurate_subtraction(&triangle.positions[0]);
//        IntersectableTriangle {
//            triangle: triangle_ptr,
//            position_0: triangle.positions[0].into(),
//            edge1: edge1.into(),
//            edge2: edge2.into(),
//        }
//    }

}

impl Intersectable for IntersectableTriangle {
    // TODO consider creating a simd version of this SoA style and benchmark
    fn intersect(&self, args: IntersectionArgs) -> bool {
        use super::intersect_util;

        let ray = args.ray;
        let record = args.record;

        let edge1 = self.edge1;
        let edge2 = self.edge2;

        let intersect_result =
            intersect_util::intersect_triangle::<MultiNum1>(
                args.ray,
                &self.position_0,
                &self.edge1,
                &self.edge2
            );

        if !intersect_result.intersected() {
            return false
        }

        if record.t < intersect_result.t {
            return false
        }

        let beta = intersect_result.beta;
        let gamma = intersect_result.gamma;
        let alpha = intersect_result.alpha();
        let t = intersect_result.t;

        let position = (ray.position + t * ray.direction.value()).into();

        *record = IntersectionRecord {
            position,
            normal: self.triangle.normals[0] * alpha +
                self.triangle.normals[1] * beta +
                self.triangle.normals[2] * gamma,
            t: t,
            shader: Some(self.triangle.shader.clone())
        };
        return true
    }

}

#[derive(Clone, Debug)]
pub struct IntersectionRecord {
    pub shader: Option<Rc<Shader>>,
    pub position: Vec3,
    pub normal: Vec3, // TODO change this to UnitVec3
    pub t: f32
}

impl IntersectionRecord {
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

