//TODO reorganize this mess
extern crate cgmath;
extern crate serde;

use self::serde::de::Error;

use utilities::codable::*;
use utilities::math::*;

#[cfg(target_feature = "avx")]
use utilities::simd::{SimdRay};
use utilities::color::*;

use super::meshutils::{MeshObject};
use super::camera::*;
use super::intersectable::*;
use super::scene_builder::{SceneBuilder, SceneSpec};
use super::shader::{Shader};
use super::bvh::*;

use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;
use std::f32;

#[derive(Debug, Deserialize)]
pub struct Light {
    pub position: CodableWrapper<Vec3>,
    pub intensity: f32
}

#[derive(Debug)]
pub struct Scene {
    pub background_color: Color3,
    pub camera: Camera,
    pub shaders: HashMap<String, Rc<Shader>>,
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

    fn intersect_intern(&self, ray: &RayUnit, obstruction_only: bool) -> IntersectionRecord {
        let index_ranges = self.intersection_accel.intersect_boxes(ray);
        let mut record = IntersectionRecord::no_intersection();

        #[cfg(target_feature = "avx")]
        let intersection_ray = &SimdRay::new(ray);
        #[cfg(not(target_feature = "avx"))]
        let intersection_ray = ray;

        'outer: for range in index_ranges {
            for i in range {
                let obj = &self.triangles[i];

                let args = IntersectionArgs {
                    ray: intersection_ray,
                    record: &mut record,
                    intersection_order: IntersectionOrderKind::FirstIntersection
                };

                let intersected = obj.intersect(args);

                if obstruction_only && intersected {
                    break 'outer; //break and return record
                }
            }
        }
        record
    }

    pub fn intersect(&self, ray: &RayUnit) -> IntersectionRecord {
        self.intersect_intern(ray, false)
    }

    ///detects an intersection between origin and destination. Not necessarily
    ///the first intersection
    ///TODO this logic doesn't belong here
    pub fn intersect_for_obstruction(
        &self, origin: Vec3, destination: Vec3
    ) -> IntersectionRecord {
        let ray = {
            let mut ray = RayUnit::new_epsilon_offset(origin, (destination - origin).unit());
            ray.t_range.end = (destination - origin).magnitude();
            ray
        };
        self.intersect_intern(&ray, true)
    }
}

