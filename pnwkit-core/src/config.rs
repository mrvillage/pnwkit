#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use std::sync::{Arc, Mutex};

#[cfg(any(feature = "async", feature = "sync"))]
use std::time::Duration;

#[cfg(feature = "subscriptions")]
use crate::socket::Socket;
use crate::{
    rate_limiter::RateLimiter,
    request::{Client, Headers},
};

#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub verified_bot_key: Option<String>,
    pub verified_bot_key_api_key: Option<String>,
    pub api_url: String,
    #[cfg(feature = "subscriptions")]
    pub socket_url: String,
    #[cfg(feature = "subscriptions")]
    pub subscribe_url: String,
    #[cfg(feature = "subscriptions")]
    pub subscription_auth_url: String,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
    #[cfg(feature = "subscriptions")]
    pub socket: Box<dyn Socket>,
    pub client: Box<dyn Client>,
    pub headers: Headers,
    pub now: fn() -> u64,
    #[cfg(feature = "async")]
    pub sleep: fn(Duration) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
    #[cfg(feature = "sync")]
    pub sleep_sync: fn(Duration) -> (),
    pub user_agent: String,
}

impl Config {
    pub fn update_headers(mut self) -> Self {
        self.headers
            .set_authorization(format!("Bearer {}", self.api_key));
        if let Some(verified_bot_key) = &self.verified_bot_key {
            self.headers.set_x_bot_key(verified_bot_key.clone());
        }
        if let Some(verified_bot_key_api_key) = &self.verified_bot_key_api_key {
            self.headers.set_x_api_key(verified_bot_key_api_key.clone());
        }
        self.headers.set_user_agent(self.user_agent.clone());
        self
    }

    pub fn set_api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self.headers
            .set_authorization(format!("Bearer {}", self.api_key));
        self
    }

    pub fn set_verified_bot_key(mut self, verified_bot_key: String) -> Self {
        self.headers.set_x_bot_key(verified_bot_key.clone());
        self.verified_bot_key = Some(verified_bot_key);
        self
    }

    pub fn set_verified_bot_key_api_key(mut self, verified_bot_key_api_key: String) -> Self {
        self.headers.set_x_api_key(verified_bot_key_api_key.clone());
        self.verified_bot_key_api_key = Some(verified_bot_key_api_key);
        self
    }

    pub fn set_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_socket_url(mut self, socket_url: String) -> Self {
        self.socket_url = socket_url;
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_subscribe_url(mut self, subscriptions_url: String) -> Self {
        self.subscribe_url = subscriptions_url;
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_subscription_auth_url(mut self, subscription_auth_url: String) -> Self {
        self.subscription_auth_url = subscription_auth_url;
        self
    }

    pub fn set_rate_limiter(mut self, rate_limiter: RateLimiter) -> Self {
        self.rate_limiter = Arc::new(Mutex::new(rate_limiter));
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_socket(mut self, socket: Box<dyn Socket>) -> Self {
        self.socket = socket;
        self
    }

    pub fn set_client(mut self, client: Box<dyn Client>) -> Self {
        self.client = client;
        self
    }

    pub fn set_now(mut self, now: fn() -> u64) -> Self {
        self.now = now;
        self
    }

    #[cfg(feature = "async")]
    pub fn set_sleep(
        mut self,
        sleep: fn(Duration) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
    ) -> Self {
        self.sleep = sleep;
        self
    }

    #[cfg(feature = "sync")]
    pub fn set_sleep_sync(mut self, sleep_sync: fn(Duration) -> ()) -> Self {
        self.sleep_sync = sleep_sync;
        self
    }

    pub fn set_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }
}
