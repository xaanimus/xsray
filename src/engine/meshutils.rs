use std::fmt;
use std::rc::Rc;

use utilities::math::Vec3;

use super::intersectable::Triangle;
use super::shader::Shader;

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
