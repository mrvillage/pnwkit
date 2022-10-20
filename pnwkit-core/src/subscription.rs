use std::{collections::VecDeque, fmt::Debug};

use tokio::sync::{Mutex, Notify};

use crate::{event::Event, Object};

#[derive(Clone, Debug)]
pub enum SubscriptionModel {
    Alliance,
    AlliancePosition,
    Bankrec,
    BBGame,
    BBTeam,
    Bounty,
    City,
    Embargo,
    Nation,
    TaxBracket,
    Trade,
    TreasureTrade,
    Treaty,
    WarAttack,
    War,
}

impl ToString for SubscriptionModel {
    fn to_string(&self) -> String {
        match self {
            Self::Alliance => "alliance",
            Self::AlliancePosition => "alliance_position",
            Self::Bankrec => "bankrec",
            Self::BBGame => "bbgame",
            Self::BBTeam => "bbteam",
            Self::Bounty => "bounty",
            Self::City => "city",
            Self::Embargo => "embargo",
            Self::Nation => "nation",
            Self::TaxBracket => "tax_bracket",
            Self::Trade => "trade",
            Self::TreasureTrade => "treasure_trade",
            Self::Treaty => "treaty",
            Self::WarAttack => "warattack",
            Self::War => "war",
        }
        .into()
    }
}

#[derive(Clone, Debug)]
pub enum SubscriptionEvent {
    Create,
    Delete,
    Update,
}

impl ToString for SubscriptionEvent {
    fn to_string(&self) -> String {
        match self {
            Self::Create => "create",
            Self::Delete => "delete",
            Self::Update => "update",
        }
        .into()
    }
}

pub struct SubscriptionQueue {
    queue: Mutex<VecDeque<Object>>,
    notify: Notify,
}

impl SubscriptionQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    pub async fn push(&self, object: Object) {
        self.queue.lock().await.push_back(object);
        self.notify.notify_waiters();
    }

    pub async fn pop(&self) -> Option<Object> {
        self.wait().await;
        self.queue.lock().await.pop_front()
    }

    pub async fn wait(&self) {
        {
            if self.queue.lock().await.is_empty() {
                self.notify.notified()
            } else {
                return;
            }
        }
        .await
    }

    pub async fn extend(&self, iter: impl Iterator<Item = Object>) {
        self.queue.lock().await.extend(iter);
        self.notify.notify_waiters();
    }
}

impl Default for SubscriptionQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Subscription {
    pub(crate) model: SubscriptionModel,
    pub(crate) event: SubscriptionEvent,
    pub(crate) filters: Object,
    pub channel: Mutex<String>,
    pub succeeded: Event,
    pub queue: SubscriptionQueue,
}

impl Subscription {
    pub fn new(
        model: SubscriptionModel,
        event: SubscriptionEvent,
        filters: Object,
        channel: String,
    ) -> Self {
        Self {
            model,
            event,
            filters,
            channel: Mutex::new(channel),
            succeeded: Event::new(),
            queue: SubscriptionQueue::new(),
        }
    }

    pub(crate) async fn set_channel(&self, channel: String) {
        *self.channel.lock().await = channel;
    }

    pub async fn next(&self) -> Option<Object> {
        self.queue.pop().await
    }

    pub async fn push(&self, object: Object) {
        self.queue.push(object).await
    }

    pub async fn extend(&self, iter: impl Iterator<Item = Object>) {
        self.queue.extend(iter).await
    }
}

impl Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("model", &self.model)
            .field("event", &self.event)
            .field("filters", &self.filters)
            .finish()
    }
}
