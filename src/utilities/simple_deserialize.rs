//TODO maybe finish, put in another crate
extern crate serde;
use self::serde::{Deserialize, Deserializer};
use self::serde::de;
use self::de::{Visitor, MapAccess};

use std::marker::PhantomData;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Formatter;

struct StringVisitor;
impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: de::Error 
    {
        Ok(v)
    }
}

struct MapStringkeyVisitor<V>(PhantomData<V>);
impl<'de, V> Visitor<'de> for MapStringkeyVisitor<V> {
    type Value = BTreeMap<String, V>;
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
        A: MapAccess<'de> {
        let mut result = BTreeMap::<String, V>::new();
        while let Some((key, value)) = map.next_entry()? {
            let key_string = key.get_string()?;
        }
        Ok(result)
    }
}

trait SimpleDeserializer<'de>: Sized {
    type ErrType;
    fn get_string(self) -> Result<String, Self::ErrType>;
    fn get_map_stringkey<E, V>(self) -> Result<BTreeMap<String, V>, E>;
}

impl<'de, D> SimpleDeserializer<'de> for D
    where D: Deserializer<'de>
{
    type ErrType = D::Error;
    fn get_string(self) -> Result<String, D::Error> {
        self.deserialize_string(StringVisitor)
    }

    fn get_map_stringkey<E, V>(self) -> Result<BTreeMap<String, V>, E> {
    }
}
