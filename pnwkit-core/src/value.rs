use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    data::{Object, ObjectVisitor},
    Variable,
};

#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
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
            Value::Int(i) => serializer.serialize_i64(*i),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::String(s) => serializer.serialize_str(s),
            Value::Variable(v) => v.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
            Value::Array(a) => a.serialize(serializer),
        }
    }
}

macro_rules! into {
    ($($v:ident, $e:ident)*) => {
        $(
            impl From<$v> for Value {
                fn from(v: $v) -> Self {
                    Self::$e(v.try_into().unwrap())
                }
            }
        )*
    };
}

macro_rules! from {
    ($($f:ident, $v:ident)*) => {
        $(
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

into!(
    bool, Bool
    i8, Int
    i16, Int
    i32, Int
    i64, Int
    i128, Int
    u8, Int
    u16, Int
    u32, Int
    u64, Int
    u128, Int
    f32, Float
    f64, Float
    String, String
    Object, Object
    VecValue, Array
);

from!(
    bool, bool
    i8, i8
    i16, i16
    i32, i32
    i64, i64
    i128, i128
    u8, u8
    u16, u16
    u32, u32
    u64, u64
    u128, u128
    f32, f32
    f64, f64
    string, String
    object, Object
    array, VecValue
);

#[cfg(feature = "uuid")]
use uuid::Uuid;

#[cfg(feature = "uuid")]
from!(uuid, Uuid);

#[cfg(feature = "bigdecimal")]
use bigdecimal::BigDecimal;

#[cfg(feature = "bigdecimal")]
from!(bigdecimal, BigDecimal);

#[cfg(feature = "time")]
use time::OffsetDateTime;

#[cfg(feature = "time")]
from!(time, OffsetDateTime);

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[cfg(feature = "chrono")]
type Chrono = DateTime<Utc>;
#[cfg(feature = "chrono")]
from!(chrono, Chrono);

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

impl<'a> From<&'a Value> for Option<&'a str> {
    fn from(v: &'a Value) -> Self {
        match v {
            Value::None => None,
            _ => Some(v.as_str().unwrap()),
        }
    }
}

impl Value {
    pub fn as_i8(&self) -> Option<i8> {
        match self {
            Value::Int(v) => Some(*v as i8),
            Value::Float(v) => Some(*v as i8),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        match self {
            Value::Int(v) => Some(*v as i16),
            Value::Float(v) => Some(*v as i16),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Int(v) => Some(*v as i32),
            Value::Float(v) => Some(*v as i32),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v),
            Value::Float(v) => Some(*v as i64),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match self {
            Value::Int(v) => Some(*v as i128),
            Value::Float(v) => Some(*v as i128),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Value::Int(v) => Some(*v as u8),
            Value::Float(v) => Some(*v as u8),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Value::Int(v) => Some(*v as u16),
            Value::Float(v) => Some(*v as u16),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Int(v) => Some(*v as u32),
            Value::Float(v) => Some(*v as u32),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Int(v) => Some(*v as u64),
            Value::Float(v) => Some(*v as u64),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_u128(&self) -> Option<u128> {
        match self {
            Value::Int(v) => Some(*v as u128),
            Value::Float(v) => Some(*v as u128),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::Int(v) => Some(*v as f32),
            Value::Float(v) => Some(*v as f32),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(v) => Some(*v as f64),
            Value::Float(v) => Some(*v),
            Value::String(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            Value::Int(v) => Some(*v != 0),
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

    #[cfg(feature = "uuid")]
    pub fn as_uuid(&self) -> Option<uuid::Uuid> {
        match self {
            Value::String(v) => uuid::Uuid::parse_str(v.as_str()).ok(),
            _ => None,
        }
    }

    #[cfg(feature = "bigdecimal")]
    pub fn as_bigdecimal(&self) -> Option<bigdecimal::BigDecimal> {
        use std::str::FromStr;

        use bigdecimal::FromPrimitive;

        match self {
            Value::Float(v) => bigdecimal::BigDecimal::from_f64(*v),
            Value::Int(v) => bigdecimal::BigDecimal::from_i64(*v),
            Value::String(v) => bigdecimal::BigDecimal::from_str(v.as_str()).ok(),
            _ => None,
        }
    }

    #[cfg(feature = "time")]
    pub fn as_time(&self) -> Option<time::OffsetDateTime> {
        match self {
            Value::String(v) => {
                if v.as_str() == "0000-00-00" || v.starts_with('-') {
                    Some(time::OffsetDateTime::UNIX_EPOCH)
                } else if v.len() == 10 {
                    time::OffsetDateTime::parse(
                        format!("{}T00:00:00Z", v).as_str(),
                        &time::format_description::well_known::Iso8601::DEFAULT,
                    )
                    .ok()
                } else {
                    time::OffsetDateTime::parse(
                        v.as_str(),
                        &time::format_description::well_known::Iso8601::DEFAULT,
                    )
                    .ok()
                }
            },
            _ => None,
        }
    }

    #[cfg(feature = "chrono")]
    pub fn as_chrono(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        match self {
            Value::String(v) => {
                if v.as_str() == "0000-00-00" || v.starts_with('-') {
                    Some(chrono::DateTime::<chrono::Utc>::MIN_UTC)
                } else if v.len() == 10 {
                    chrono::DateTime::parse_from_rfc3339(format!("{}T00:00:00Z", v).as_str())
                        .ok()
                        .map(|v| v.with_timezone(&chrono::Utc))
                } else {
                    chrono::DateTime::parse_from_rfc3339(v.as_str())
                        .ok()
                        .map(|v| v.with_timezone(&chrono::Utc))
                }
            },
            _ => None,
        }
    }

    pub fn string_to_value(&self) -> Option<Value> {
        if let Some(s) = self.as_str() {
            serde_json::from_str::<Value>(s).ok()
        } else {
            Some(self.clone())
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
}
