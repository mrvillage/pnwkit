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
    pub(crate) paginate_name: bool,
}

impl Field {
    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn set_alias(mut self, alias: String) -> Self {
        self.alias = Some(alias);
        self
    }

    pub fn set_argument(self, name: String, value: Value) -> Self {
        self.arguments.insert(name, value);
        self
    }

    pub fn add_field(mut self, field: FieldType) -> Self {
        self.fields.push(field);
        self
    }

    pub fn add_field_node(mut self, field: Field) -> Self {
        self.fields.push(FieldType::Node(field));
        self
    }

    pub fn add_field_leaf(mut self, field: &str) -> Self {
        self.fields.push(FieldType::Leaf(field.into()));
        self
    }

    pub fn will_paginate(mut self) -> Self {
        self.paginate = true;
        self.paginate_name = true;
        self
    }

    pub fn is_paginated(mut self) -> Self {
        self.paginate = true;
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

    pub(crate) fn tree_will_paginate(&self) -> bool {
        if self.paginate {
            return true;
        }
        for field in &self.fields {
            match field {
                FieldType::Node(field) => {
                    if field.tree_will_paginate() {
                        return true;
                    }
                },
                FieldType::Leaf(_) => {},
            }
        }
        false
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
            paginate_name: false,
        }
    }
}

pub fn field(name: &str) -> Field {
    Field::default().set_name(name.into())
}

pub fn field_as(name: &str, alias: &str) -> Field {
    Field::default()
        .set_name(name.into())
        .set_alias(alias.into())
}
