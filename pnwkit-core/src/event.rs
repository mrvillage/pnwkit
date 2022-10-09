use std::sync::Arc;

use tokio::sync::{Notify, RwLock};

pub struct Event {
    pub(crate) state: RwLock<EventInner>,
    pub(crate) notify: Arc<Notify>,
}

pub struct EventInner(bool);

impl Event {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(EventInner(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn wait(&self) -> bool {
        let state = {
            let state = self.state.read().await;
            state.0
        };
        if !state {
            self.notify.notified().await;
        }
        true
    }

    pub async fn set(&self) {
        {
            let mut state = self.state.write().await;
            if !(state.0) {
                state.0 = true;
            }
        }
        self.notify.notify_waiters();
    }

    pub async fn clear(&self) {
        let mut state = self.state.write().await;
        state.0 = false;
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}
