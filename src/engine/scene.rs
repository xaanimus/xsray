//TODO reorganize this mess
extern crate cgmath;
extern crate serde;

use self::serde::de::Error;

use utilities::codable::*;
use utilities::math::*;
use utilities::color::*;

use super::camera::*;
use super::intersectable::*;
use super::scene_builder::{SceneBuilder, SceneSpec};
use super::shader::{Shader};
use super::bvh_accelerator::*;

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

#[derive(Debug, Deserialize)]
pub struct Light {
    pub position: CodableWrapper<Vec3>,
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
    pub triangles: Vec<IntersectableTriangle>
}

impl_deserialize!(Scene, |deserializer| {
    let spec = SceneSpec::deserialize(deserializer)?;
    let builder = spec.to_builder()
        .map_err(|scene_error| D::Error::custom(scene_error.0))?;
    Ok(Scene::new_from_builder(builder))
});

impl Scene {
    pub fn new_from_builder(builder: SceneBuilder) -> Scene {
        let mut bb_triangles: Vec<TriangleWithAABoundingBox> = builder.meshes.into_iter()
            .flat_map(|mesh: MeshObject| mesh.triangles.iter()
                      .map(|triangle| TriangleWithAABoundingBox::new_from_triangle(triangle))
                      .collect::<Vec<TriangleWithAABoundingBox>>())
            .collect();

        let intersection_accel = BVHAccelerator::new(&mut bb_triangles);
        let intersectable_triangles: Vec<IntersectableTriangle> = bb_triangles.iter()
            .map(|bb_triangle| IntersectableTriangle::new_from_triangle(&bb_triangle.triangle))
            .collect();

        Scene {
            background_color: builder.background_color.get(),
            camera: builder.camera,
            shaders: {
                let mut shaders = HashMap::<String, Rc<Shader>>::new();
                for (key, value) in builder.shaders.iter() {
                    shaders.insert(key.clone(), value.get());
                }
                shaders
            },
            lights: builder.lights,
            intersection_accel: intersection_accel,
            triangles: intersectable_triangles
        }
    }

    pub fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        let indices = self.intersection_accel.intersect_boxes(ray, false);
        let mut record = IntersectionRecord::no_intersection();
        for i in indices {
            let obj = &self.triangles[i];
            obj.intersect(ray, &mut record);
        }
        record
    }

    ///detects an intersection between origin and destination. Not necessarily
    ///the first intersection
    ///TODO this logic doesn't belong here
    pub fn intersect_for_obstruction(
        &self, origin: Vec3, destination: Vec3
    ) -> IntersectionRecord {
        //TODO optimize for obstruction detection
        let ray = {
            let mut ray = RayBase::new_epsilon_offset(origin, (destination - origin).unit());
            ray.t_range.end = (destination - origin).magnitude();
            ray
        };

        let indices = self.intersection_accel.intersect_boxes(&ray, false);
        let mut record = IntersectionRecord::no_intersection();
        for i in indices {
            let obj = &self.triangles[i];
            if obj.intersect(&ray, &mut record) {
                return record
            }
        }
        record
    }
}

