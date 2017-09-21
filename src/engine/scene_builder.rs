use super::scene::*;
use super::color::*;
use super::camera::*;
use super::shader::*;
use std::collections::HashMap;
use std::rc::Rc;

pub struct SceneBuilder {
    pub background_color: Color3,
    pub camera: Camera,
    pub shaders: HashMap<String, Rc<Shader>>,
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
            background_color: Color3::new(0.0,0.0,0.0),
            camera: Camera::new_default(),
            shaders: HashMap::new(),
            meshes: Vec::new(),
            lights: Vec::new()
        }
    }

    pub fn build(self) -> Scene {
        Scene::new_from_builder(self)
    }

    builder_param!(background_color, Color3);
    builder_param!(camera, Camera);
    builder_param!(shaders, HashMap<String, Rc<Shader>>);
    builder_param!(meshes, Vec<MeshObject>);
    builder_param!(lights, Vec<Light>);
}

