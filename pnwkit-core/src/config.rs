#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use std::sync::{Arc, Mutex};

#[cfg(feature = "subscriptions")]
use crate::socket::Socket;
use crate::{
    rate_limiter::RateLimiter,
    request::{Client, Headers},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) api_key: String,
    pub(crate) verified_bot_key: Option<String>,
    pub(crate) verified_bot_key_api_key: Option<String>,
    pub(crate) api_url: String,
    #[cfg(feature = "subscriptions")]
    pub(crate) socket_url: String,
    #[cfg(feature = "subscriptions")]
    pub(crate) subscribe_url: String,
    #[cfg(feature = "subscriptions")]
    pub(crate) subscription_auth_url: String,
    pub(crate) rate_limiter: Arc<Mutex<RateLimiter>>,
    #[cfg(feature = "subscriptions")]
    pub(crate) socket: Box<dyn Socket>,
    pub(crate) client: Box<dyn Client>,
    pub(crate) headers: Headers,
    now: fn() -> u64,
    #[cfg(feature = "async")]
    pub(crate) sleep: fn(u64) -> Pin<Box<dyn Future<Output = ()>>>,
    #[cfg(feature = "sync")]
    pub(crate) sleep_sync: fn(u64) -> (),
}

impl Config {
    pub fn set_api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = api_key;
        self.headers
            .set_authorization(format!("Bearer {}", self.api_key));
        self
    }

    pub fn set_verified_bot_key(&mut self, verified_bot_key: String) -> &mut Self {
        self.headers.set_x_bot_key(verified_bot_key.clone());
        self.verified_bot_key = Some(verified_bot_key);
        self
    }

    pub fn set_verified_bot_key_api_key(&mut self, verified_bot_key_api_key: String) -> &mut Self {
        self.headers.set_x_api_key(verified_bot_key_api_key.clone());
        self.verified_bot_key_api_key = Some(verified_bot_key_api_key);
        self
    }

    pub fn set_api_url(&mut self, api_url: String) -> &mut Self {
        self.api_url = api_url;
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_socket_url(&mut self, socket_url: String) -> &mut Self {
        self.socket_url = socket_url;
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_subscribe_url(&mut self, subscriptions_url: String) -> &mut Self {
        self.subscribe_url = subscriptions_url;
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_subscription_auth_url(&mut self, subscription_auth_url: String) -> &mut Self {
        self.subscription_auth_url = subscription_auth_url;
        self
    }

    pub fn set_rate_limiter(&mut self, rate_limiter: RateLimiter) -> &mut Self {
        self.rate_limiter = Arc::new(Mutex::new(rate_limiter));
        self
    }

    #[cfg(feature = "subscriptions")]
    pub fn set_socket(&mut self, socket: Box<dyn Socket>) -> &mut Self {
        self.socket = socket;
        self
    }

    pub fn set_client(&mut self, client: Box<dyn Client>) -> &mut Self {
        self.client = client;
        self
    }

    pub fn set_now(&mut self, now: fn() -> u64) -> &mut Self {
        self.now = now;
        self
    }

    #[cfg(feature = "async")]
    pub fn set_sleep(&mut self, sleep: fn(u64) -> Pin<Box<dyn Future<Output = ()>>>) -> &mut Self {
        self.sleep = sleep;
        self
    }

    #[cfg(feature = "sync")]
    pub fn set_sleep_sync(&mut self, sleep_sync: fn(u64) -> ()) -> &mut Self {
        self.sleep_sync = sleep_sync;
        self
    }
}
