use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{event::Event, Kit, Subscription};

// subscriptions: Arc<RwLock<DashMap<String, Arc<Subscription>>>>
#[async_trait]
pub trait Socket: Debug + Send + Sync + 'static {
    async fn init(&self, kit: Kit);

    async fn get_socket_id(&self) -> String;

    fn get_established(&'_ self) -> &'_ Event;

    fn get_connected(&'_ self) -> &'_ Event;

    async fn add_subscription(&self, subscription: Arc<Subscription>);

    async fn remove_subscription(&self, subscription: Arc<Subscription>);

    async fn get_subscription(&self, channel: String) -> Option<Arc<Subscription>>;

    async fn send(&self, data: String) -> Result<(), String>;

    async fn connect_ref(&self) -> Result<(), String>;

    async fn connect(self) -> Result<(), String>;

    async fn reconnect(&self) -> Result<(), String>;

    async fn ping_pong(self);

    async fn call_later_pong(self);

    fn start_ping_pong_task(&self);
}
