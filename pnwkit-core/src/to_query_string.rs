use crate::{Object, Value};

pub trait ToQueryString {
    fn to_query_string(&self) -> String;
}

impl ToQueryString for Object {
    fn to_query_string(&self) -> String {
        self.iter()
            .map(|i| {
                let (key, value) = i.pair();
                format!("{}={}", key, value.to_query_string())
            })
            .collect::<Vec<String>>()
            .join("&")
    }
}

impl ToQueryString for Value {
    fn to_query_string(&self) -> String {
        match self {
            Value::Bool(v) => v.to_string(),
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::String(v) => v.to_string(),
            Value::Object(v) => v.to_query_string(),
            Value::Array(v) => v
                .iter()
                .map(|i| i.to_query_string())
                .collect::<Vec<String>>()
                .join(","),
            Value::None => "".to_string(),
            Value::Variable(_) => "".to_string(),
        }
    }
}
