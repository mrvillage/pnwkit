use crate::{Field, Variable};

#[derive(Clone, Debug)]
pub enum QueryType {
    Mutation,
    Query,
}

#[derive(Clone, Debug)]
pub struct Query {
    pub(crate) query_type: QueryType,
    pub(crate) fields: Vec<Field>,
}

impl Query {
    pub fn new(query_type: QueryType) -> Self {
        Self {
            query_type,
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub(crate) fn get_variables(&self) -> Vec<Variable> {
        self.fields
            .iter()
            .flat_map(|f| f.get_variables())
            .collect::<Vec<Variable>>()
    }

    pub(crate) fn valid(&self) -> Result<(), String> {
        if self.fields.is_empty() {
            return Err("no fields".into());
        }
        for field in &self.fields {
            if let Err(msg) = field.valid() {
                return Err(format!("invalid field: {}", msg));
            }
        }
        Ok(())
    }
}
