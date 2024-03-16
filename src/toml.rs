use basic_toml::Error;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::map::{Entry, Map};
use serde_json::{Number, Value};
use std::fmt;

pub(crate) fn from_str<T>(document: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let result = basic_toml::from_str::<T>(document);
    let error = match result {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };

    let result = basic_toml::from_str::<Toml>(document);
    if let Ok(toml) = result {
        if let Ok(value) = T::deserialize(toml.0) {
            return Ok(value);
        }
    }

    Err(error)
}

struct Toml(Value);

impl<'de> Deserialize<'de> for Toml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TomlVisitor).map(Toml)
    }
}

struct TomlVisitor;

impl<'de> Visitor<'de> for TomlVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid TOML value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
        Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(String::from(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }

        Ok(Value::Array(vec))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut values = Map::new();

        while let Some((key, value)) = visitor.next_entry::<String, Value>()? {
            match values.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(mut entry) => {
                    merge(entry.get_mut(), value);
                }
            }
        }

        Ok(Value::Object(values))
    }
}

fn merge(a: &mut Value, b: Value) {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}
