use super::scene::*;
use super::color::*;
use super::camera::*;
use super::shader::*;
use std::collections::HashMap;
use std::rc::Rc;

//TODO refactor this hack. maybe put this in config
#[derive(Debug)]
pub struct SceneSpec {
    pub background_color: Color3,
    pub camera: Camera,
    pub shaders: HashMap<String, Rc<Shader>>,
    pub meshes: Vec<MeshObject>,
    pub lights: Vec<Light>
}
