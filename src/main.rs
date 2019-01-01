#![target_feature(enable = "avx")]

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate regex;

#[macro_use]
mod utilities;
mod engine;

use std::env;
use std::fs::File;
use std::path::Path;
use std::io::Read;

use self::regex::Regex;

use engine::renderer::Config;

pub fn load_yml_config_from_string(directory_prefix: &str, text: &str)
                                   -> Result<Config, serde_yaml::Error>
{
    let re = Regex::new(r"'\./(?P<path>.*)'").unwrap();
    let text_with_relative_dirs: String = re.replace_all(
        text,
        format!("{}/$path", directory_prefix).as_str()
    ).into();
    let config: Config = serde_yaml::from_str(text_with_relative_dirs.as_str()).unwrap();
    Ok(config)
}

fn main() {
    //load scene file
    let mut arguments = env::args();
    arguments.next();
    let filename = arguments.next().expect("no filename provided");
    let filepath = Path::new(filename.as_str());

    let mut file = File::open(filepath).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    let directory_string = filepath.parent().unwrap()
        .to_str().unwrap();
    println!("using directory prefix (for loading assets): {}", directory_string);
    println!("rendering {}", filename);

    //load scene
    let render_config = load_yml_config_from_string(directory_string, s.as_str())
        .unwrap();
    
    let buffer = render_config.render();
    let output_filename = filename.clone() + ".png";
    buffer.save(output_filename).unwrap();
}
