extern crate cgmath;

use super::math::*;
use super::color::*;
use super::camera::*;
use super::scene_builder::SceneBuilder;
use super::shader::{Shader};
use super::bvh_accelerator::{BVHAccelerator, HasAABoundingBox, AABoundingBox};
use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;
use std::f32;

pub trait Intersectable {
    /// check for intersection between ray and surface.
    /// returns IntersectionRecord with t=inf if no intersection
    fn intersect(&self, ray: &RayUnit) -> IntersectionRecord;
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
    fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        self.triangle.intersect(ray)
    }
}

impl HasAABoundingBox for BVHTriangleWrapper {
    fn aa_bounding_box(&self) -> &AABoundingBox {
        &self.bounding_box
    }
}

impl Intersectable for Triangle {
    fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {

        //using barymetric coordinates to intersect with this triangle
        // vectors a, b, and c are the 0, 1, and 2 vertices for this triangle
        let a_col_1 = self.positions[0] - self.positions[1]; //a - b
        let a_col_2 = self.positions[0] - self.positions[2]; //a - c
        let a_col_3 = ray.direction.vec(); //d
        let b_col = self.positions[0] - ray.position;

        //compute determinant of A
        let mut a_mat = Matrix3::from_cols(a_col_1, a_col_2, *a_col_3);
        let det_a = a_mat.determinant();

        if det_a == 0. {
            return IntersectionRecord::no_intersection()
        }

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
            //TODO on construct check normals normalized
            IntersectionRecord {
                position: ray.position + t * ray.direction.vec(),
                normal: self.normals[0] * alpha + self.normals[1] * beta + self.normals[2] * gamma,
                t: t,
                shader: Some(self.shader.clone())
            }
        } else {
            IntersectionRecord::no_intersection()
        }
    }

}

pub struct MeshInfo {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub triangles: Vec<([usize; 3], [usize; 3])>, // vector of indices for (position, normal)
}

pub struct MeshObject {
    pub triangles: Vec<Triangle>,
    pub shader: Rc<Shader>
}

impl MeshObject {
    pub fn new(mesh_info: &MeshInfo, shader: &Rc<Shader>) -> Option<MeshObject> {

        let mut mesh_object = MeshObject {
            triangles: Vec::<Triangle>::new(),
            shader: shader.clone()
        };

        {
            for &(positions, normals) in &mesh_info.triangles {
                //positions
                if let (Some(pos0), Some(pos1), Some(pos2),
                        Some(norm0), Some(norm1), Some(norm2)) = (
                    mesh_info.positions.get(positions[0]),
                    mesh_info.positions.get(positions[1]),
                    mesh_info.positions.get(positions[2]),
                    mesh_info.normals.get(normals[0]),
                    mesh_info.normals.get(normals[1]),
                    mesh_info.normals.get(normals[2]),
                ) {
                    let triangle = Triangle {
                        positions: [*pos0, *pos1, *pos2],
                        normals: [*norm0, *norm1, *norm2],
                        shader: shader.clone()
                    };
                    mesh_object.triangles.push(triangle);
                } else {
                    return None
                }
            }
        }

        Some(mesh_object)
    }
}

impl fmt::Debug for MeshObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Shader")
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
}

#[derive(Debug)]
pub struct Light {
    pub position: Vec3,
    pub intensity: f32
}

#[derive(Debug)]
pub struct Scene {
    pub background_color: Color3,
    pub camera: Camera,
    pub shaders: HashMap<String, Rc<Shader>>, //delet this
    //pub meshes: Vec<MeshObject>, //refactor code to maybe include ref to object intersected with
    pub lights: Vec<Light>,
    pub intersection_accel: BVHAccelerator<BVHTriangleWrapper>,
}

impl Scene {
    pub fn new_from_builder(builder: SceneBuilder) -> Scene {
        let triangle_wrappers = builder.meshes.into_iter()
            .fold(vec![], |acc, mesh| {
                mesh.triangles.into_iter()
                    .map(|triangle: Triangle| BVHTriangleWrapper::new(triangle))
                    .collect()
            });
        Scene {
            background_color: builder.background_color,
            camera: builder.camera,
            shaders: builder.shaders,
            lights: builder.lights,
            intersection_accel: BVHAccelerator::new_into(triangle_wrappers.into_boxed_slice())
        }
    }

    pub fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        self.intersection_accel.intersect(ray)
    }

    ///detects an intersection between origin and destination. Not necessarily
    ///the first intersection
    pub fn intersect_for_obstruction(
        &self, origin: Vec3, destination: Vec3
    ) -> IntersectionRecord {
        //TODO optimize for shadow detection
        let ray = {
            let mut ray = RayUnit::new_shadow(origin, (destination - origin).unit());
            ray.t_range.end = (destination - origin).magnitude();
            ray
        };
        self.intersect(&ray)
    }
}
