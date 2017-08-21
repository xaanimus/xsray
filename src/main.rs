
mod engine;
mod config_loader;
mod utilities;

use std::env;
use std::fs::File;
use std::io::Read;

fn main() {

    //load scene file
    let arguments : Vec<String> = env::args().collect();
    let filename = &arguments[1];
    let mut file = File::open(filename).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    //load scene
    let render_config = config_loader::load_config_from_string(s.as_str()).unwrap();

    let buffer = render_config.render();
    buffer.save("render_out.png").unwrap();

}
