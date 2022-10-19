use crate::{
    field::FieldType,
    query::{Query, QueryType},
    variable, Field, Object, Value, Variable, VariableType,
};

pub(crate) trait Resolve {
    fn resolve(&self) -> String;
}

impl Resolve for Query {
    fn resolve(&self) -> String {
        let mut vars = self.get_variables();
        if self.fields.iter().any(|f| f.tree_will_paginate())
            && !vars.iter().any(|v| v.name == "__page")
        {
            vars.push(variable("__page", VariableType::Int));
        }
        format!(
            "{}{} {{ {} }}",
            self.query_type.resolve(),
            vars.resolve(),
            self.fields
                .iter()
                .map(|f| f.resolve())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

impl Resolve for Vec<Variable> {
    fn resolve(&self) -> String {
        if self.is_empty() {
            return "".into();
        }
        format!(
            "({})",
            self.iter()
                .map(|v| v.resolve())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Resolve for QueryType {
    fn resolve(&self) -> String {
        match self {
            QueryType::Mutation => "mutation",
            QueryType::Query => "query",
        }
        .into()
    }
}

impl Resolve for Field {
    fn resolve(&self) -> String {
        let name = if self.paginate_name {
            format!("__paginate:{}", self.name)
        } else if let Some(alias) = &self.alias {
            format!("{}:{}", alias, self.name)
        } else {
            self.name.clone()
        };
        let inner = if self.paginate {
            format!(
                "data{{{}}} paginatorInfo {{__typename count currentPage firstItem hasMorePages lastItem lastPage perPage total}}",
            self.fields.resolve())
        } else {
            self.fields.resolve()
        };
        if self.paginate && !self.arguments.contains_key("page") {
            self.arguments.insert(
                "page".into(),
                Value::Variable(variable("__page", VariableType::Int)),
            );
        };
        let arguments = self.arguments.resolve();
        format!("{}{}{{__typename {}}}", name, arguments, inner)
    }
}

impl Resolve for FieldType {
    fn resolve(&self) -> String {
        match self {
            FieldType::Leaf(f) => f.clone(),
            FieldType::Node(f) => f.resolve(),
        }
    }
}

impl Resolve for Variable {
    fn resolve(&self) -> String {
        format!("${}: {}", self.name, self.variable_type.resolve())
    }
}

impl Resolve for VariableType {
    fn resolve(&self) -> String {
        match self {
            VariableType::Int => "Int",
            VariableType::String => "String",
        }
        .into()
    }
}

impl Resolve for Object {
    fn resolve(&self) -> String {
        if self.is_empty() {
            return "".into();
        }
        format!(
            "({})",
            self.iter()
                .map(|a| format!("{}: {}", a.key(), a.value().resolve()))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Resolve for Value {
    fn resolve(&self) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Variable(v) => format!("${}", v.name.clone()),
            Self::None => "null".into(),
            Self::Bool(b) => b.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Object(v) => format!("{{ {} }}", v.resolve()),
            Self::Array(v) => format!(
                "[{}]",
                v.iter()
                    .map(|i| i.resolve())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl Resolve for Vec<FieldType> {
    fn resolve(&self) -> String {
        self.iter()
            .map(|f| f.resolve())
            .collect::<Vec<String>>()
            .join(" ")
    }
}
