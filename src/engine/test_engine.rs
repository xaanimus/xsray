
use std::rc::Rc;
use super::scene::*;
use super::math::*;
use super::color::*;
use super::shader::DiffuseShader;
use super::bvh_accelerator::HasAABoundingBox;

//#[test]
//fn test_triangle_wrapper() {
//    let triangle = Triangle {
//        positions: [
//            Rc::new(Vec3::new(0.0, 0.0, 0.0)),
//            Rc::new(Vec3::new(3.0, 0.0, 0.0)),
//            Rc::new(Vec3::new(1.5, 3.0, 0.0)),
//        ],
//        normals: [
//            Rc::new(Vec3::new(1.0, 1.0, 1.0)),
//            Rc::new(Vec3::new(1.0, 1.0, 1.0)),
//            Rc::new(Vec3::new(1.0, 1.0, 1.0)),
//        ],
//        shader: Rc::new(DiffuseShader::new(Color3::new(1.0, 1.0, 1.0)))
//    };
//    let wrapper = BVHTriangleWrapper::new(triangle);
//    let bb = wrapper.aa_bounding_box();
//    println!("bb {:?}", bb);
//}
