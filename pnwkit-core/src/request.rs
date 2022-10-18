use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub body: Option<String>,
    pub headers: Option<Headers>,
    pub content_type: Option<ContentType>,
}

impl Request {
    pub fn new(
        method: Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
        content_type: Option<ContentType>,
    ) -> Self {
        Self {
            method,
            url,
            body,
            headers,
            content_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

#[derive(Debug, Default, Clone)]
pub struct Headers {
    pub authorization: String,
    pub x_api_key: Option<String>,
    pub x_bot_key: Option<String>,
    pub user_agent: String,
}

impl Headers {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_authorization(&mut self, authorization: String) {
        self.authorization = authorization;
    }

    pub(crate) fn set_x_api_key(&mut self, x_api_key: String) {
        self.x_api_key = Some(x_api_key);
    }

    pub(crate) fn set_x_bot_key(&mut self, x_bot_key: String) {
        self.x_bot_key = Some(x_bot_key);
    }

    pub(crate) fn set_user_agent(&mut self, user_agent: String) {
        self.user_agent = user_agent;
    }
}

#[derive(Clone, Debug)]
pub enum ContentType {
    Json,
    Form,
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Json
    }
}

impl ToString for ContentType {
    fn to_string(&self) -> String {
        match self {
            Self::Json => "application/json".to_string(),
            Self::Form => "application/x-www-form-urlencoded".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub(crate) status: u16,
    pub(crate) body: String,
    pub(crate) x_ratelimit_reset: Option<u64>,
}

impl Response {
    pub fn new(status: u16, body: String, x_ratelimit_reset: Option<u64>) -> Self {
        Self {
            status,
            body,
            x_ratelimit_reset,
        }
    }
}

pub type ResponseResult = Result<Response, String>;

#[async_trait]
pub trait Client: Debug + Send + Sync + 'static {
    #[cfg(any(feature = "async", feature = "subscriptions"))]
    async fn request(&self, request: &Request) -> ResponseResult;

    #[cfg(feature = "sync")]
    fn request_sync(&self, request: &Request) -> ResponseResult;
}
