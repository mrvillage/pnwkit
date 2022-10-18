use pnwkit_core::{async_trait, Method};
#[cfg(any(feature = "sync", feature = "async", feature = "subscriptions"))]
use pnwkit_core::{Request, Response, ResponseResult};

#[derive(Clone, Debug)]
pub struct Client {
    #[cfg(any(feature = "async", feature = "subscriptions"))]
    client: reqwest::Client,
    #[cfg(feature = "sync")]
    sync_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new() -> Self {
        Client {
            #[cfg(any(feature = "async", feature = "subscriptions"))]
            client: reqwest::Client::new(),
            #[cfg(feature = "sync")]
            sync_client: reqwest::blocking::Client::new(),
        }
    }
}

trait ToReqwestMethod {
    fn to_reqwest_method(&self) -> reqwest::Method;
}

impl ToReqwestMethod for pnwkit_core::Method {
    fn to_reqwest_method(&self) -> reqwest::Method {
        match self {
            Method::Delete => reqwest::Method::DELETE,
            Method::Get => reqwest::Method::GET,
            Method::Patch => reqwest::Method::PATCH,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
        }
    }
}

macro_rules! setup_request {
    ($client:ident, $request:ident) => {{
        let req = $client.request($request.method.to_reqwest_method(), $request.url.clone());
        let req = if let Some(body) = &$request.body {
            req.body(body.clone())
        } else {
            req
        };
        let req = if let Some(headers) = &$request.headers {
            let mut req = req;
            if !headers.authorization.is_empty() {
                req = req.header("Authorization", headers.authorization.clone());
            }
            if let Some(x_api_key) = &headers.x_api_key {
                req = req.header("X-API-Key", x_api_key.clone());
            }
            if let Some(x_bot_key) = &headers.x_bot_key {
                req = req.header("X-Bot-Key", x_bot_key.clone());
            }
            req = req.header("User-Agent", headers.user_agent.clone());
            req
        } else {
            req
        };
        let req = if let Some(content_type) = &$request.content_type {
            req.header("Content-Type", content_type.to_string())
        } else {
            req
        };
        req
    }};
}

#[async_trait]
impl pnwkit_core::Client for Client {
    #[cfg(any(feature = "async", feature = "subscriptions"))]
    async fn request(&self, request: &Request) -> ResponseResult {
        let client = &self.client;
        let req = setup_request!(client, request);
        let res = req.send().await;
        match res {
            Ok(res) => {
                let status = res.status().as_u16();
                let x_ratelimit_reset = res
                    .headers()
                    .get("X-Ratelimit-Reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                let body = res.text().await;
                match body {
                    Ok(body) => Ok(Response::new(status, body, x_ratelimit_reset)),
                    Err(err) => Err(err.to_string()),
                }
            },
            Err(err) => Err(err.to_string()),
        }
    }

    #[cfg(feature = "sync")]
    fn request_sync(&self, request: &Request) -> ResponseResult {
        let client = &self.sync_client;
        let req = setup_request!(client, request);
        let res = req.send();
        match res {
            Ok(res) => {
                let status = res.status().as_u16();
                let x_ratelimit_reset = res
                    .headers()
                    .get("X-Ratelimit-Reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                let body = res.text();
                match body {
                    Ok(body) => Ok(Response::new(status, body, x_ratelimit_reset)),
                    Err(err) => Err(err.to_string()),
                }
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
