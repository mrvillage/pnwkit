use dashmap::DashMap;
use serde::{ser::SerializeMap, Serialize};

use crate::{query::Query, Value};

#[derive(Clone, Debug)]
pub enum VariableType {
    Int,
    String,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub(crate) name: String,
    pub(crate) variable_type: VariableType,
}

impl Serialize for Variable {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unreachable!()
    }
}

pub fn variable(name: &str, variable_type: VariableType) -> Variable {
    Variable {
        name: name.into(),
        variable_type,
    }
}

#[derive(Debug)]
pub struct Variables(pub(crate) DashMap<String, Value>);

impl Variables {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(DashMap::with_capacity(capacity))
    }

    pub fn for_query(query: &Query) -> Self {
        let mut vars = query
            .get_variables()
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<String>>();
        vars.sort();
        vars.dedup();
        Self::with_capacity(vars.len())
    }

    pub fn set(&self, name: String, value: Value) -> &Self {
        if self.0.contains_key(&name) {
            self.0.remove(&name);
        }
        self.0.insert(name, value);
        self
    }

    pub fn get(&self, name: String) -> Option<Value> {
        self.0.get(&name).map(|v| v.value().clone())
    }

    pub(crate) fn valid(&self, names: Vec<String>) -> Result<(), String> {
        for name in names {
            if name.is_empty() {
                return Err("empty variable name".into());
            }
            if !self.0.contains_key(&name) {
                return Err(format!("missing variable: {}", name));
            }
        }
        Ok(())
    }

    pub(crate) fn page_init(&self) {
        if self.0.contains_key("__page") {
            return;
        }
        self.set("__page".into(), Value::Int(1));
    }
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Variables {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for pair in self.0.iter() {
            let (key, value) = pair.pair();
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}
