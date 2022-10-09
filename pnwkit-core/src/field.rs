use crate::{data::Object, Value, Variable};
use dashmap::DashMap;

#[derive(Clone, Debug)]
pub enum FieldType {
    Node(Field),
    Leaf(String),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub(crate) name: String,
    pub(crate) alias: Option<String>,
    pub(crate) arguments: Object,
    pub(crate) fields: Vec<FieldType>,
    pub(crate) paginate: bool,
}

impl Field {
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_alias(&mut self, alias: String) -> &mut Self {
        self.alias = Some(alias);
        self
    }

    pub fn set_argument(&mut self, name: String, value: Value) -> &mut Self {
        self.arguments.insert(name, value);
        self
    }

    pub fn add_field(&mut self, field: FieldType) -> &mut Self {
        self.fields.push(field);
        self
    }

    pub fn set_paginate(&mut self, paginate: bool) -> &mut Self {
        self.paginate = paginate;
        self
    }

    pub(crate) fn get_variables(&self) -> Vec<Variable> {
        let mut vars = Vec::new();
        for i in self.arguments.iter() {
            if let Value::Variable(variable) = i.value() {
                vars.push(variable.clone());
            }
        }
        for field in &self.fields {
            match field {
                FieldType::Node(field) => vars.extend(field.get_variables()),
                FieldType::Leaf(_) => {},
            }
        }
        vars
    }

    pub(crate) fn valid(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Field name cannot be empty".into());
        }
        Ok(())
    }
}

impl Default for Field {
    fn default() -> Self {
        Self {
            name: String::new(),
            alias: None,
            arguments: DashMap::new(),
            fields: Vec::new(),
            paginate: false,
        }
    }
}

pub fn field(name: String) -> Field {
    Field::default().set_name(name).to_owned()
}

pub fn field_as(name: String, alias: String) -> Field {
    let mut field = Field::default();
    field.set_name(name).set_alias(alias);
    field
}
