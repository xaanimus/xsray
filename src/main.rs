#![feature(target_feature)]
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod engine;
mod utilities;

use std::env;
use std::fs::File;
use std::io::Read;

use engine::renderer::Config;

pub fn load_yml_config_from_string(text: &str) -> Result<Config, serde_yaml::Error> {
    let config: Config = serde_yaml::from_str(text).unwrap();
    Ok(config)
}

fn main() {
    //load scene file
    let mut arguments = env::args();
    arguments.next();
    let filename = arguments.next().expect("no filename provided");

    let mut file = File::open(filename).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    //load scene
    //let render_config = config_loader::load_config_from_string(s.as_str()).unwrap();
    let render_config = load_yml_config_from_string(s.as_str())
        .unwrap();
    
    let buffer = render_config.render();
    buffer.save("render_out.png").unwrap();
}
