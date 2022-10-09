use std::fmt::Debug;

use async_trait::async_trait;

use crate::{clone_box, event::Event};

#[async_trait]
pub trait Socket: Debug + SocketClone {
    fn get_socket_id(&self) -> String;

    fn get_established(&self) -> Event;

    async fn send(&self, event: String, data: String) -> Result<(), String>;
}

clone_box!(Socket, SocketClone);
