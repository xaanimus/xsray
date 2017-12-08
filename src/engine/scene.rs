extern crate cgmath;

use super::math::*;
use super::color::*;
use super::camera::*;
use super::intersectable::*;
use super::scene_builder::SceneBuilder;
use super::shader::{Shader};
use super::bvh_accelerator::{BVHAccelerator};
use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;
use std::f32;

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
        write!(f, "MeshObject")
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
    pub intersection_accel: BVHAccelerator,
    pub triangle_wrappers: Vec<BVHTriangleWrapper>
}

impl Scene {
    pub fn new_from_builder(builder: SceneBuilder) -> Scene {
        let mut triangle_wrappers: Vec<BVHTriangleWrapper> = builder.meshes.into_iter()
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
            intersection_accel: BVHAccelerator::new(&mut triangle_wrappers),
            triangle_wrappers: triangle_wrappers
        }
    }

    pub fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        let indices = self.intersection_accel.intersect_boxes(ray, false);
        let mut max_intersection = IntersectionRecord::no_intersection();
        for i in indices {
            let obj = &self.triangle_wrappers[i];
            let intersection = obj.intersect(ray);
            if intersection.t < max_intersection.t {
                max_intersection = intersection;
            }
        }
        max_intersection
    }

    ///detects an intersection between origin and destination. Not necessarily
    ///the first intersection
    ///TODO this logic doesn't belong here
    pub fn intersect_for_obstruction(
        &self, origin: Vec3, destination: Vec3
    ) -> IntersectionRecord {
        //TODO optimize for shadow detection
        let ray = {
            let mut ray = RayUnit::new_shadow(origin, (destination - origin).unit());
            ray.t_range.end = (destination - origin).magnitude();
            ray
        };

        let indices = self.intersection_accel.intersect_boxes(&ray, true);
        for i in indices {
            let obj = &self.triangle_wrappers[i];
            let intersection = obj.intersect(&ray);
            if intersection.t.is_finite() {
                return intersection
            }
        }

        IntersectionRecord::no_intersection()
    }
}
