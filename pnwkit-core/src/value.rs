use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    data::{Object, ObjectVisitor},
    Variable,
};

#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Variable(Variable),
    Object(Object),
    Array(Vec<Value>),
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor::new())
    }
}

#[derive(Clone, Copy, Debug)]
struct ValueVisitor {}

impl ValueVisitor {
    fn new() -> Self {
        Self {}
    }
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::None)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        Ok(ObjectVisitor::new().visit_map(map)?.into())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec: Vec<Value> = Vec::new();
        while let Some(value) = seq.next_element::<Value>()? {
            vec.push(value);
        }
        Ok(vec.into())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::None)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::None => serializer.serialize_none(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Int(i) => serializer.serialize_i32(*i),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::String(s) => serializer.serialize_str(s),
            Value::Variable(v) => v.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
            Value::Array(a) => a.serialize(serializer),
        }
    }
}

macro_rules! from {
    ($($f:ident, $v:ident, $e:ident)*) => {
        $(
            impl From<$v> for Value {
                fn from(v: $v) -> Self {
                    Self::$e(v.try_into().unwrap())
                }
            }

            impl From<Value> for $v {
                fn from(v: Value) -> Self {
                    paste::paste! {
                        v.[<as_ $f>]().unwrap()
                    }
                }
            }

            impl From<&Value> for $v {
                fn from(v: &Value) -> Self {
                    paste::paste! {
                        v.[<as_ $f>]().unwrap()
                    }
                }
            }

            impl From<Value> for Option<$v> {
                fn from(v: Value) -> Self {
                    match v {
                        Value::None => None,
                        _ => Some(paste::paste! { v.[<as_ $f>]().unwrap() }),
                    }
                }
            }

            impl From<&Value> for Option<$v> {
                fn from(v: &Value) -> Self {
                    match v {
                        Value::None => None,
                        _ => Some(paste::paste! { v.[<as_ $f>]().unwrap() }),
                    }
                }
            }
        )*
    };
}

type VecValue = Vec<Value>;

from!(
    bool, bool, Bool
    i32, i32, Int
    i64, i64, Int
    u64, u64, Int
    f64, f64, Float
    string, String, String
    object, Object, Object
    array, VecValue, Array
);

impl From<Variable> for Value {
    fn from(v: Variable) -> Self {
        Self::Variable(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.into())
    }
}

impl<'a> From<&'a Value> for &'a str {
    fn from(v: &'a Value) -> Self {
        v.as_str().unwrap()
    }
}

impl Value {
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Int(v) => Some(*v as u64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<Object> {
        match self {
            Value::Object(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn parse_object(&self) -> Option<Object> {
        match self {
            Value::Object(v) => Some(v.clone()),
            Value::String(v) => serde_json::from_str(v).ok(),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<Vec<Value>> {
        match self {
            Value::Array(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn string_to_value(&self) -> Option<Value> {
        if let Some(s) = self.as_str() {
            serde_json::from_str::<Value>(s).ok()
        } else {
            None
        }
    }
}
