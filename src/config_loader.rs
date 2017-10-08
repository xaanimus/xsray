//TODO refactor this file to make less ugly
use std::error::Error;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;
use std::rc::Rc;

extern crate obj;
extern crate json;
use engine::scene_builder::*;
use engine::scene::{Scene, MeshObject, MeshInfo, Light};
use utilities::math::*;
use engine::camera::*;
use engine::shader::*;
use engine::renderer::*;


#[derive(Debug)]
pub enum ConfigError<'a, 'b> {
    ConfigLoadError(Option<&'a Error>),
    ConfigJsonError(&'b str),
    MiscError(&'b str)
}

impl<'a, 'b> Error for ConfigError<'a, 'b> {
    fn description(&self) -> &str {
        match self {
            &ConfigError::ConfigLoadError(_) => "ConfigLoadError",
            &ConfigError::ConfigJsonError(_) => "Json Parse Error",
            &ConfigError::MiscError(s) => s
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &ConfigError::ConfigLoadError(err) => err,
            _ => None,
        }
    }
}

impl<'a, 'b> Display for ConfigError<'a, 'b> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let &ConfigError::ConfigJsonError(msg) = self {
            write!(f, "{}", msg)?;
        }
        write!(f, "{}", self.description())
    }
}


fn parse_vec3(obj: &json::JsonValue) -> Option<Vec3> {
    match obj {
        &json::JsonValue::Array(ref arr) if arr.len() == 3 => {
            if let (Some(x), Some(y), Some(z)) = (arr[0].as_f32(),
                                                  arr[1].as_f32(),
                                                  arr[2].as_f32()) {
                Some(Vec3::new(x,y,z))
            } else {
                None
            }
        }
        _ => None
    }
}

fn parse_camera(obj: &json::JsonValue, aspect_ratio: f32) -> Option<Camera> {
    if let (Some(position), Some(direction), Some(up),
            Some(plane_distance), Some(plane_width)) = (
        parse_vec3(&obj["position"]),
        parse_vec3(&obj["direction"]),
        parse_vec3(&obj["up"]),
        obj["plane_distance"].as_f32(),
        obj["plane_width"].as_f32())
    {
        Some(Camera::new(position, direction, up, plane_width,
                         plane_width / aspect_ratio, plane_distance))
    } else {
        None
    }
}

fn parse_shader_object(obj: &json::JsonValue) -> Option<Rc<Shader>> {
    match obj["type"].as_str() {
        Some("diffuse") => {
            parse_vec3(&obj["color"]).map(|color| {
                Rc::new(DiffuseShader::new(color)) as Rc<Shader>
            })
        },
        _ => None
    }
}

fn parse_shader_array(obj: &json::JsonValue) -> Option<HashMap<String, Rc<Shader>>> {
    //check that the obj is an array
    match obj {
        &json::JsonValue::Object(ref shader_object_map) => {
            let mut shaders = HashMap::<String, Rc<Shader>>::new();
            let mut iterator = shader_object_map.iter();
            while let Some((key, shader_obj)) = iterator.next() {
                //try to parse each shader object
                if let Some(shader) = parse_shader_object(shader_obj) {
                    shaders.insert(key.to_string(), shader);
                }
            }
            Some(shaders)
        },
        _ => None
    }
}

fn polygons_to_triangles(polys: &Vec<obj::raw::object::Polygon>)
                         -> Option<Vec<([usize; 3], [usize; 3])>>
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
                println!("Fatal error: polygons must be triangles!");
            },
            _ => return None
        }
    }

    Some(collect)
}

fn parse_mesh_info(filepath: &str) -> Result<MeshInfo, ConfigError> {
    if let Ok(reader) = File::open(filepath).map(|file| BufReader::new(file)) {
        if let Ok(object) = obj::raw::parse_obj(reader) {
            if let Some(triangles) = polygons_to_triangles(&object.polygons) {
                Ok(MeshInfo {
                    positions: object.positions.iter()
                        .map(|pos| Vec3::new(pos.0, pos.1, pos.2)).collect(),
                    normals: object.normals.iter()
                        .map(|pos| Vec3::new(pos.0, pos.1, pos.2)).collect(),
                    triangles: triangles
                })
            } else {
                println!("obj mesh not ok");
                Err(ConfigError::MiscError("Obj mesh not ok"))
            }
        } else {
            Err(ConfigError::MiscError("Could not parse obj file"))
        }
    } else {
        Err(ConfigError::ConfigJsonError("Could not load obj file"))
    }
}

//given an array of mesh objects, parse and convert to MeshObject
fn parse_mesh_array<'a,'b,'c>(obj: &json::JsonValue, shaders: &'a HashMap<String, Rc<Shader>>)
                           -> Result<Vec<MeshObject>, ConfigError<'b, 'c>>
{
    match obj {
        &json::JsonValue::Array(ref vec) => {
            let mut meshes = Vec::<MeshObject>::new();
            for mesh_obj in vec {
                if let (Some(mesh_info_src), Some( Some(shader))) = (
                    mesh_obj["src"].as_str(),
                    mesh_obj["shader"].as_str().map(|shader_id|{
                        shaders.get(shader_id)
                    }))
                {
                    match parse_mesh_info(mesh_info_src) {
                        Ok(mesh_info) => {
                            if let Some(mesh) = MeshObject::new(&mesh_info, shader) {
                                meshes.push(mesh);
                            } else {
                                return Err(ConfigError::MiscError("Could not construct a valid mesh"))
                            }
                        },
                        Err(_) => return Err(ConfigError::MiscError("bad"))
                    }
                } else {
                    return Err(ConfigError::MiscError("Could not parse mesh info"))
                }
            }

            Ok(meshes)
        },
        _ => Err(ConfigError::ConfigJsonError("expected mesh array"))
    }
}

fn parse_lights<'a, 'b>(obj: &json::JsonValue) -> Result<Vec<Light>, ConfigError<'a, 'b>> {
    match obj {
        &json::JsonValue::Array(ref vec) => {
            let mut lights = Vec::<Light>::new();
            for light_obj in vec {
                if let (Some(pos_vec), Some(intensity)) = (
                    parse_vec3(&light_obj["position"]),
                    light_obj["intensity"].as_f32())
                {
                    lights.push(Light{position: pos_vec, intensity: intensity});
                } else {
                    return Err(ConfigError::MiscError("Parse lights error"))
                }
            }
            Ok(lights)
        },
        _ => Err(ConfigError::ConfigJsonError("While parsing lights, expected array"))
    }
}

pub fn load_config_from_string(text: &str) -> Result<Config, ConfigError> {
    match json::parse(text) {
        Err(_) => Err(ConfigError::ConfigLoadError(Option::None)),
        Ok(obj) => {

            // parse render_settings object
            let settings =
                if let (Some(width), Some(height), Some(exp),) = (
                    obj["render_settings"]["resolution_width"].as_i32(),
                    obj["render_settings"]["resolution_height"].as_i32(),
                    obj["render_settings"]["exposure"].as_f32()
                ) {
                    Ok(RenderSettings {
                        resolution_width: width,
                        resolution_height: height,
                        exposure: exp,
                    })
                } else {
                    Err(ConfigError::ConfigJsonError("config missing fields in render_settings"))
                }?;

            // Parse scene object
            let aspect_ratio = (settings.resolution_width / settings.resolution_height) as f32;
            let sc_obj = &obj["scene"];
            let scene: Scene = if let (Some(backgnd_color), Some(camera)) = (
                parse_vec3(&sc_obj["background_color"]),
                parse_camera(&sc_obj["camera"], aspect_ratio)
            ) {
                let shaders = parse_shader_array(&sc_obj["shaders"])
                    .unwrap_or(HashMap::<String, Rc<Shader>>::new());
                {
                    let meshes = try!(parse_mesh_array(&sc_obj["meshes"], &shaders));
                    let lights = try!(parse_lights(&sc_obj["lights"]));
                    Ok(SceneBuilder::new()
                        .background_color(backgnd_color)
                        .camera(camera)
                        .shaders(shaders)
                        .meshes(meshes)
                        .lights(lights)
                        .build()
                    )
                }
            } else {
                Err(ConfigError::ConfigJsonError("config missing fields in scene"))
            }?;

            Ok(Config::new(
                settings,
                scene
            ))
        }
    }
}
