use std::collections::VecDeque;

#[cfg(any(feature = "async", feature = "sync"))]
use crate::Kit;

use crate::{query::Query, variable::Variables, Value};

#[derive(Debug)]
pub struct PaginatorInfo {
    count: i32,
    current_page: i32,
    first_item: i32,
    has_more_pages: bool,
    last_item: i32,
    last_page: i32,
    per_page: i32,
    total: i32,
}

impl PaginatorInfo {
    fn update(&mut self, value: &Value) {
        if let Value::Object(o) = value {
            for i in o.iter() {
                let (k, v) = i.pair();
                match k.as_str() {
                    "count" => self.count = v.as_i32().unwrap(),
                    "currentPage" => self.current_page = v.as_i32().unwrap(),
                    "firstItem" => self.first_item = v.as_i32().unwrap(),
                    "hasMorePages" => self.has_more_pages = v.as_bool().unwrap(),
                    "lastItem" => self.last_item = v.as_i32().unwrap(),
                    "lastPage" => self.last_page = v.as_i32().unwrap(),
                    "perPage" => self.per_page = v.as_i32().unwrap(),
                    "total" => self.total = v.as_i32().unwrap(),
                    _ => {},
                }
            }
        }
    }
}

impl From<&Value> for PaginatorInfo {
    fn from(value: &Value) -> Self {
        let mut info = Self {
            count: 0,
            current_page: 0,
            first_item: 0,
            has_more_pages: false,
            last_item: 0,
            last_page: 0,
            per_page: 0,
            total: 0,
        };
        info.update(value);
        info
    }
}

#[derive(Debug)]
pub struct Paginator {
    pub paginator_info: Option<PaginatorInfo>,
    query: Query,
    variables: Variables,
    queue: VecDeque<Value>,
}

impl Paginator {
    pub fn new(query: Query) -> Self {
        let variables = Variables::with_capacity(1);
        variables.set("__page".into(), 0.into());
        Self {
            paginator_info: None,
            query,
            variables,
            queue: VecDeque::new(),
        }
    }

    pub fn with_capacity(query: Query, capacity: u16) -> Self {
        let variables = Variables::with_capacity(1);
        variables.set("__page".into(), 0.into());
        Self {
            paginator_info: None,
            query,
            variables,
            queue: VecDeque::with_capacity(capacity as usize),
        }
    }

    pub fn with_variables(query: Query, variables: Variables) -> Self {
        variables.set("__page".into(), 0.into());
        Self {
            paginator_info: None,
            query,
            variables,
            queue: VecDeque::new(),
        }
    }

    pub fn with_capacity_and_variables(query: Query, capacity: u16, variables: Variables) -> Self {
        variables.set("__page".into(), 0.into());
        Self {
            paginator_info: None,
            query,
            variables,
            queue: VecDeque::with_capacity(capacity as usize),
        }
    }

    #[cfg(feature = "async")]
    pub async fn next(&mut self, kit: &Kit) -> Option<Value> {
        if self.queue.is_empty() && (self.fill(kit).await).is_err() {
            return None;
        }
        self.queue.pop_front()
    }

    #[cfg(feature = "sync")]
    pub fn next_sync(&mut self, kit: &Kit) -> Option<Value> {
        if self.queue.is_empty() && self.fill_sync(kit).is_err() {
            return None;
        }
        self.queue.pop_front()
    }

    #[cfg(feature = "async")]
    pub async fn fill(&mut self, kit: &Kit) -> Result<(), String> {
        match self.page() {
            Ok(end) => {
                if end {
                    return Ok(());
                }
            },
            Err(e) => return Err(e),
        }
        let result = kit.get_with_variables(&self.query, &self.variables).await?;
        let result = result.inner();
        let result = result.get("__paginate").unwrap();
        self.result(result.value());
        Ok(())
    }

    #[cfg(feature = "sync")]
    pub fn fill_sync(&mut self, kit: &Kit) -> Result<(), String> {
        match self.page() {
            Ok(end) => {
                if end {
                    return Ok(());
                }
            },
            Err(e) => return Err(e),
        }
        let result = kit.get_with_variables_sync(&self.query, &self.variables)?;
        let result = result.inner();
        let result = result.get("__paginate").unwrap();
        self.result(result.value());
        Ok(())
    }

    #[cfg(any(feature = "async", feature = "sync"))]
    fn page(&self) -> Result<bool, String> {
        if self.paginator_info.is_some() && !self.paginator_info.as_ref().unwrap().has_more_pages {
            return Ok(true);
        }
        let page = self.variables.get("__page".into()).unwrap();
        let page = match page {
            Value::Int(i) => i,
            _ => {
                if self.paginator_info.is_none() {
                    return Err("invalid paginator variable".into());
                } else {
                    self.paginator_info.as_ref().unwrap().current_page
                }
            },
        };
        self.variables.set("__page".into(), (page + 1).into());
        Ok(false)
    }

    #[cfg(any(feature = "async", feature = "sync"))]
    fn result(&mut self, result: &Value) {
        if let Value::Object(o) = result {
            let paginator_info = o.get("paginatorInfo").unwrap();
            let paginator_info = paginator_info.value();
            if let Some(p) = &mut self.paginator_info {
                p.update(paginator_info)
            } else {
                self.paginator_info = Some(paginator_info.into())
            }

            let data = o.get("data").unwrap();
            let data = data.value();
            if let Value::Array(l) = data {
                for i in l.iter() {
                    self.queue.push_back(i.clone());
                }
            }
        };
    }
}
