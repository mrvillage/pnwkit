use dashmap::DashMap;
use serde::{de::Visitor, Deserialize};

use crate::Value;

pub type Object = DashMap<String, Value>;

type Errors = Vec<Error>;

#[derive(Clone, Debug, Deserialize)]
pub struct Error {
    pub(crate) message: String,
}

// type Extensions = Vec<Extension>;

// #[derive(Clone, Debug, Deserialize)]
// pub struct Extension {
//     pub(crate) key: String,
//     pub(crate) version: u8,
// }

#[derive(Clone, Debug, Deserialize)]
pub struct QueryReturn {
    pub(crate) errors: Option<Errors>,
    pub(crate) data: Option<Data>,
    // pub(crate) extensions: Option<Extensions>,
}

#[derive(Clone, Debug)]
pub struct Data(pub Object);

impl Data {
    pub fn inner(self) -> Object {
        self.0
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Data, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(deserializer.deserialize_map(ObjectVisitor::new())?))
    }
}

#[derive(Clone, Debug)]
pub struct ObjectVisitor {}

impl ObjectVisitor {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'de> Visitor<'de> for ObjectVisitor {
    type Value = Object;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let map = Object::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

#[cfg(feature = "subscriptions")]
#[derive(Clone, Debug, Deserialize)]
pub struct SubscriptionAuthData {
    pub(crate) auth: String,
}
