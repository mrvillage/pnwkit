use std::sync::{Arc, Mutex};

#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use pnwkit_core::{Headers, Kit, RateLimiter};

use crate::{client::Client, socket::Socket};

#[cfg(feature = "async")]
fn sleep(duration: std::time::Duration) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(tokio::time::sleep(duration))
}

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
}

impl Config {
    pub fn new() -> Self {
        Config {
            api_key: String::new(),
            verified_bot_key: None,
            verified_bot_key_api_key: None,
            api_url: String::new(),
            #[cfg(feature = "subscriptions")]
            socket_url: String::new(),
            #[cfg(feature = "subscriptions")]
            subscribe_url: String::new(),
            #[cfg(feature = "subscriptions")]
            subscription_auth_url: String::new(),
        }
    }

    pub fn set_api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = api_key;
        self
    }

    pub fn set_verified_bot_key(&mut self, verified_bot_key: String) -> &mut Self {
        self.verified_bot_key = Some(verified_bot_key);
        self
    }

    pub fn set_verified_bot_key_api_key(&mut self, verified_bot_key_api_key: String) -> &mut Self {
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

    pub fn to_kit(self) -> Kit {
        let now = || 1;
        let config = pnwkit_core::Config {
            api_key: self.api_key,
            verified_bot_key: self.verified_bot_key,
            verified_bot_key_api_key: self.verified_bot_key_api_key,
            api_url: self.api_url,
            #[cfg(feature = "subscriptions")]
            socket_url: self.socket_url,
            #[cfg(feature = "subscriptions")]
            subscribe_url: self.subscribe_url,
            #[cfg(feature = "subscriptions")]
            subscription_auth_url: self.subscription_auth_url,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(now))),
            #[cfg(feature = "subscriptions")]
            socket: Box::new(Socket::new()),
            client: Box::new(Client::new()),
            headers: Headers::new(),
            now,
            #[cfg(feature = "async")]
            sleep,
            #[cfg(feature = "sync")]
            sleep_sync: std::thread::sleep,
            user_agent: format!("pnwkit-rs/{}", env!("CARGO_PKG_VERSION")),
        };
        Kit::new(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
