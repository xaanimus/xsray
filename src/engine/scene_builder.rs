extern crate obj;

use utilities::codable::*;
use utilities::math::*;

use super::transformable::*;
use super::meshutils::*;
use super::scene::*;
use super::color::*;
use super::camera::*;
use super::shader::*;

use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;
use std::rc::Rc;
use std::error::Error;


pub struct SceneError(pub String);

fn polygons_to_triangles(polys: &Vec<obj::raw::object::Polygon>)
                         -> Result<Vec<([usize; 3], [usize; 3])>, SceneError>
{
    let mut collect = Vec::<([usize; 3], [usize; 3])>::new();
    for poly in polys {
        match poly {
            &obj::raw::object::Polygon::PN(ref vn_vec) if vn_vec.len() == 3 => {
                let ind = ([vn_vec[0].0, vn_vec[1].0, vn_vec[2].0],
                           [vn_vec[0].1, vn_vec[1].1, vn_vec[2].1]);
                collect.push(ind);
            },
            //TODO make this less ugly
            &obj::raw::object::Polygon::PN(ref vn_vec) if vn_vec.len() != 3 => {
                return Err(SceneError("Fatal error: polygons must be triangles!".into()));
            },
            _ => return Err(SceneError("an error occured while converting polys to tris".into()))
        }
    }

    Ok(collect)
}

fn parse_mesh_info(filepath: &str) -> Result<MeshInfo, SceneError> {
    let reader = File::open(filepath).map(|file| BufReader::new(file))
        .map_err(|err| SceneError(err.description().into()))?;

    let object = obj::raw::parse_obj(reader)
        .map_err(|err| SceneError(err.description().into()))?;

    let triangles = polygons_to_triangles(&object.polygons)?;

    Ok(MeshInfo {
        positions: object.positions.iter()
            .map(|pos| Vec3::new(pos.0, pos.1, pos.2)).collect(),
        normals: object.normals.iter()
            .map(|pos| Vec3::new(pos.0, pos.1, pos.2)).collect(),
        triangles: triangles
    })
}

#[derive(Deserialize)]
pub struct MeshSpec {
    pub src: String,
    pub shader: String,
    pub transformations: Option<TransformationSpecList>
}

#[derive(Deserialize)]
pub struct SceneSpec {
    pub background_color: CodableWrapper<Color3>,
    pub camera: Camera,
    pub shaders: HashMap<String, CodableWrapper<Rc<Shader>>>,
    pub meshes: Vec<MeshSpec>,
    pub lights: Vec<Light>
}

impl SceneSpec {
    fn make_meshes(&self) -> Result<Vec<MeshObject>, SceneError> {
        let mut result_meshes: Vec<MeshObject> = vec![];
        for mesh_spec in &self.meshes {
            let shader = self.shaders.get(&mesh_spec.shader)
                .ok_or(SceneError("Shader not found".into()))?;
            let mesh_info = parse_mesh_info(mesh_spec.src.as_str())?;
            let mut mesh = MeshObject::new(&mesh_info, &shader.get().clone())
                .ok_or(SceneError("MeshObject failed to build.".into()))?;
            if let Some(transformations) = mesh_spec.transformations.as_ref()
                .map(transformation_list_to_mat4) {
                mesh.transform_in_place(&transformations);
            }
            result_meshes.push(mesh);
        }
        Ok(result_meshes)
    }

    pub fn to_builder(self) -> Result<SceneBuilder, SceneError> {
        let meshes = self.make_meshes()?;
        Ok(SceneBuilder::new()
           .background_color(self.background_color)
           .camera(self.camera)
           .shaders(self.shaders)
           .meshes(meshes)
           .lights(self.lights))
    }
}

pub struct SceneBuilder {
    pub background_color: CodableWrapper<Color3>,
    pub camera: Camera,
    pub shaders: HashMap<String, CodableWrapper<Rc<Shader>>>,
    pub meshes: Vec<MeshObject>,
    pub lights: Vec<Light>
}

macro_rules! builder_param {
    ($param:ident, $typ:ty) => (
        pub fn $param(mut self, $param: $typ) -> Self {
            self.$param = $param;
            self
        }
    )
}

impl SceneBuilder {
    pub fn new() -> SceneBuilder {
        SceneBuilder {
            background_color: Color3::new(0.0,0.0,0.0).into(),
            camera: Camera::new_default(),
            shaders: HashMap::new(),
            meshes: Vec::new(),
            lights: Vec::new()
        }
    }

    pub fn build(self) -> Scene {
        Scene::new_from_builder(self)
    }

    builder_param!(background_color, CodableWrapper<Color3>);
    builder_param!(camera, Camera);
    builder_param!(shaders, HashMap<String, CodableWrapper<Rc<Shader>>>);
    builder_param!(meshes, Vec<MeshObject>);
    builder_param!(lights, Vec<Light>);
}

