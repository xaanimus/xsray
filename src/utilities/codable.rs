extern crate serde;
extern crate cgmath;

pub use self::serde::{Deserialize, Deserializer};
pub use self::serde::de::{Visitor, MapAccess};
use super::math::{Vec3, UnitVec3, HasUnit};

impl<T> From<T> for CodableWrapper<T> {
    fn from(value: T) -> CodableWrapper<T> {
        CodableWrapper(value)
    }
}

#[derive(Debug)]
pub struct CodableWrapper<T>(pub T);

impl<T: Clone> CodableWrapper<T> {
    pub fn get(&self) -> T {self.0.clone()}
}

impl<T> CodableWrapper<T> {
    pub fn get_ref(&self) -> &T {&self.0}
}

impl<'de> Deserialize<'de> for CodableWrapper<Vec3> {
    fn deserialize<D>(deserializer: D) -> Result<CodableWrapper<Vec3>, D::Error>
        where D: Deserializer<'de> {
        let arr: [f32;3] = (<[f32;3] as Deserialize>::deserialize(deserializer))?;
        Ok(CodableWrapper(Vec3 {
            x: arr[0],
            y: arr[1],
            z: arr[2]
        }))
    }
}

impl<'de> Deserialize<'de> for UnitVec3 {
    fn deserialize<D>(deserializer: D) -> Result<UnitVec3, D::Error>
        where D: Deserializer<'de> {
        let arr: [f32;3] = (<[f32;3] as Deserialize>::deserialize(deserializer))?;
        Ok(Vec3::new(arr[0], arr[1], arr[2]).unit())
    }
}
