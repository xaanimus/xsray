extern crate cgmath;

use self::cgmath::Rad;

use utilities::math::{Matrix4, Vec3, One};
use utilities::codable::CodableWrapper;

pub trait Transformable {
    fn transform_in_place(&mut self, transform: &Matrix4);

    fn transform(&self, transform: &Matrix4) -> Self
        where Self : Clone
    {
        let mut copy = self.clone();
        copy.transform_in_place(transform);
        copy
    }
}

#[derive(Deserialize)]
pub enum SingleTransformSpec {
    Translate(CodableWrapper<Vec3>),
    Scale(CodableWrapper<Vec3>),
    RotateX(f32),
    RotateY(f32),
    RotateZ(f32)
}

impl SingleTransformSpec {
    pub fn to_matrix4(&self) -> Matrix4 {
        use self::SingleTransformSpec::*;
        match *self {
            Translate(CodableWrapper(t)) => Matrix4::from_translation(t),
            Scale(CodableWrapper(s)) =>
                Matrix4::from_nonuniform_scale(s.x, s.y, s.z),
            RotateX(x) => Matrix4::from_angle_x(Rad(x)),
            RotateY(y) => Matrix4::from_angle_y(Rad(y)),
            RotateZ(z) => Matrix4::from_angle_z(Rad(z)),
        }
    }
}

pub type TransformationSpecList = Vec<SingleTransformSpec>;

pub fn transformation_list_to_mat4(transform_list: &TransformationSpecList) -> Matrix4 {
    transform_list.iter()
        .fold(Matrix4::one(), |acc, new_transform|
        acc * new_transform.to_matrix4())
}
