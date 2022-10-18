use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc};

use tokio::sync::{Mutex, RwLock};

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

pub type SubscriptionCallback = fn(&Object) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>;

pub struct Subscription {
    pub(crate) model: SubscriptionModel,
    pub(crate) event: SubscriptionEvent,
    pub(crate) filters: Object,
    pub channel: Mutex<String>,
    pub succeeded: Event,
    pub callbacks: Arc<RwLock<Vec<SubscriptionCallback>>>,
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
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_callback(&self, callback: SubscriptionCallback) {
        self.callbacks.write().await.push(callback);
    }

    pub(crate) async fn set_channel(&self, channel: String) {
        *self.channel.lock().await = channel;
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
