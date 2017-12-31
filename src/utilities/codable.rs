extern crate serde;
extern crate cgmath;

pub use self::serde::{Deserialize, Deserializer};
pub use self::serde::de::{Visitor, MapAccess};
use super::math::{Vec3, UnitVec3, HasUnit};

macro_rules! impl_deserialize {
    ($typename:ty, $converter:expr) => {
        impl<'de> Deserialize<'de> for $typename {
            fn deserialize<D>(deserializer: D) -> Result<$typename, D::Error>
                where D: Deserializer<'de>
            {
                $converter(deserializer)
            }
        }
    }
}

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

impl_deserialize!(CodableWrapper<Vec3>, |deserializer| {
    let arr: [f32;3] = (<[f32;3] as Deserialize>::deserialize(deserializer))?;
    Ok(CodableWrapper(Vec3 {
        x: arr[0],
        y: arr[1],
        z: arr[2]
    }))
});

impl_deserialize!(UnitVec3, |deserializer| {
    let arr: [f32;3] = (<[f32;3] as Deserialize>::deserialize(deserializer))?;
    Ok(Vec3::new(arr[0], arr[1], arr[2]).unit())
});
