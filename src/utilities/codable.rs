extern crate serde;
extern crate cgmath;

use self::cgmath::Vector3;
pub use self::serde::{Deserialize, Deserializer};
pub use self::serde::de::{Visitor, MapAccess};
use self::serde::de;
use super::math::{Vec3, UnitVec3};

use std::marker::PhantomData;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::fmt;

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
        Ok(
            UnitVec3::new(&Vec3::new(arr[0], arr[1], arr[2]))
        )
    }
}

//pub struct DeserializerWrapper<'de, D>(pub D, PhantomData<&'de D>)
//    where D: Deserializer<'de> + 'de;
//
//impl<'de, D> Deserialize<'de> for DeserializerWrapper<'de, D>
//    where D: Deserializer<'de> + 'de
//{
//    fn deserialize<Ds>(deserializer: Ds) -> Result<DeserializerWrapper<'de, D>, Ds::Error>
//        where Ds: Deserializer<'de> {
//        Ok(DeserializerWrapper(deserializer, PhantomData::default()))
//    }
//}

//pub struct MapStringkeyVisitor<'de, V>(PhantomData<&'de V>)
//    where V: Deserializer<'de> + 'de;
//
//impl<'de, V> MapStringkeyVisitor<'de, V>
//    where V: Deserializer<'de> + 'de
//{
//    pub fn new() -> MapStringkeyVisitor<'de, V> {
//        MapStringkeyVisitor::<'de, V>(PhantomData::default())
//    }
//}
//
//impl<'de, V> Visitor<'de> for MapStringkeyVisitor<'de, V>
//    where V: Deserializer<'de> + 'de
//{
//    type Value = BTreeMap<String, V>;
//    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
//        formatter.write_str("a map")
//    }
//
//    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
//        where A: MapAccess<'de>
//    {
//        let mut result = BTreeMap::<String, V>::new();
//        while let Some((key, value)) = map.next_entry()? {
//            let key_string = String::deserialize(key)?;
//            result.insert(key_string, value);
//        }
//        Ok(result)
//    }
//}
