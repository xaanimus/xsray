extern crate cgmath;

use super::misc::*;
use super::camera::*;
use super::shader::{Shader};
use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;
use std::f32;

pub trait Intersectable {
    /// check for intersection between ray and surface.
    /// consider making this return an optional reference to intersected
    /// surface
    fn intersect(&self, ray: &Ray, record: &mut IntersectionRecord);
}

pub struct Triangle {
    //TODO pub testing only
    pub positions: [Rc<Vec3>; 3],
    pub normals: [Rc<Vec3>; 3]
}

impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray, record: &mut IntersectionRecord) {

        //using barymetric coordinates to intersect with this triangle
        // vectors a, b, and c are the 0, 1, and 2 vertices for this triangle
        let a_col_1 = *self.positions[0] - *self.positions[1]; //a - b
        let a_col_2 = *self.positions[0] - *self.positions[2]; //a - c
        let a_col_3 = &ray.direction; //d
        let b_col = *self.positions[0] - ray.position;

        //compute determinant of A
        let mut a_mat = Matrix3::from_cols(a_col_1, a_col_2, *a_col_3);
        let det_a = a_mat.determinant();

        if det_a == 0. {
            record.t = f32::NAN;
            return ()
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
            record.normal = *self.normals[0] * alpha + *self.normals[1] * beta +
                *self.normals[2] * gamma;
            record.position = ray.position + t * ray.direction;
            record.t = t;
        } else {
            record.t = f32::NAN;
            return ()
        }
    }
}

pub struct MeshInfo {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub triangles: Vec<([usize; 3], [usize; 3])>, // vector of indices for (position, normal)
}

pub struct MeshObject {
    pub positions: Vec<Rc<Vec3>>,
    pub normals: Vec<Rc<Vec3>>,
    pub triangles: Vec<Triangle>,
    pub shader: Rc<Shader>
}

impl MeshObject {
    pub fn new(mesh_info: &MeshInfo, shader: &Rc<Shader>) -> Option<MeshObject> {

        let mut mesh_object = MeshObject {
            positions: mesh_info.positions.iter()
                .map(|pos| Rc::new(Vec3::new(pos.x, pos.y, pos.z))).collect(),
            normals: mesh_info.normals.iter()
                .map(|pos| Rc::new(Vec3::new(pos.x, pos.y, pos.z))).collect(),
            triangles: Vec::<Triangle>::new(),
            shader: shader.clone()
        };

        {
            let pos_arr = &mesh_object.positions;
            let norm_arr = &mesh_object.normals;

            for &(positions, normals) in &mesh_info.triangles {
                //positions
                if let (Some(pos0), Some(pos1), Some(pos2),
                        Some(norm0), Some(norm1), Some(norm2)) = (
                    pos_arr.get(positions[0]),
                    pos_arr.get(positions[1]),
                    pos_arr.get(positions[2]),
                    norm_arr.get(normals[0]),
                    norm_arr.get(normals[1]),
                    norm_arr.get(normals[2]),
                ) {
                    let triangle = Triangle {
                        positions: [pos0.clone(), pos1.clone(), pos2.clone()],
                        normals: [norm0.clone(), norm1.clone(), norm2.clone()]
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
    pub position: Vec3,
    pub normal: Vec3,
    pub t: f32
}

impl IntersectionRecord {
    pub fn uninitialized() -> IntersectionRecord {
        IntersectionRecord {
            position: Vec3{x: 0., y: 0., z: 0.},
            normal: Vec3{x: 0., y: 0., z: 0.},
            t: f32::NAN //consider making this infty
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
    pub shaders: HashMap<String, Rc<Shader>>,
    pub meshes: Vec<MeshObject>,
    pub lights: Vec<Light>
}

impl Scene {
    pub fn intersect(&self, ray: &Ray) -> Option<(IntersectionRecord, Rc<Shader>)> {
        if self.meshes.len() == 0 {
            None
        } else {
            let mut shader : Option<Rc<Shader>> = None;
            let mut max_record = IntersectionRecord::uninitialized();
            let mut record = IntersectionRecord::uninitialized();
            max_record.t = f32::INFINITY;
            for obj in &self.meshes {
                for tri in &obj.triangles {
                    tri.intersect(ray, &mut record);
                    if record.t < max_record.t { //intersection detected
                        shader = Some(obj.shader.clone());
                        max_record = record.clone();
                    }
                }
            }

            if f32::is_finite(max_record.t) {
                Some((max_record, shader.unwrap()))
            } else {
                None
            }
        }
    }

    ///detects an intersection between origin and destination. Not necessarily
    ///the first intersection
    pub fn intersect_for_obstruction(&self, origin: Vec3,
                                     destination: Vec3) -> Option<(IntersectionRecord, Rc<Shader>)> {
        let ray = Ray::new_shadow(origin, (destination - origin).normalize());
        let max_t = (destination - origin).magnitude();
        if self.meshes.len() == 0 {
            None
        } else {
            let mut record = IntersectionRecord::uninitialized();
            record.t = f32::INFINITY;
            for obj in &self.meshes {
                for tri in &obj.triangles {
                    tri.intersect(&ray, &mut record);
                    if record.t <= max_t {
                        return Some((record, obj.shader.clone()));
                    }
                }
            }
            None
        }
    }
}
