
mod engine;
mod config_loader;

/*
fn main() {

//load scene
let s = include_str!("../test_scene.json");
let render_config = config_loader::load_config_from_string(s).unwrap();

let buffer = render_config.render();
buffer.save("render_out.png").unwrap();

}
*/

use std::rc::Rc;
use engine::scene::Intersectable;

fn main() {
    let ray = engine::misc::Ray::new(
        engine::Vec3::new(0., 0., 10.),
        engine::Vec3::new(-0.1, 0., -1.));

    let tri = engine::scene::Triangle {
        positions: [Rc::new(engine::Vec3::new(-1., -1., 0.)),
                    Rc::new(engine::Vec3::new(1., -1., 0.)),
                    Rc::new(engine::Vec3::new(0., 1., 0.))],
        normals: [Rc::new(engine::Vec3::new(-1., -1., 0.)),
                  Rc::new(engine::Vec3::new(1., -1., 0.)),
                  Rc::new(engine::Vec3::new(0., 1., 0.))],
    };

    let mut record = engine::scene::IntersectionRecord::uninitialized();

    tri.intersect(&ray, &mut record);

    println!("done");
}
