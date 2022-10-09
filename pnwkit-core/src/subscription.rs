use std::fmt::Debug;

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

pub type SubscriptionCallback = Box<dyn Fn(Object) + Send + Sync>;

pub struct Subscription {
    pub(crate) model: SubscriptionModel,
    pub(crate) event: SubscriptionEvent,
    pub(crate) filters: Object,
    pub(crate) channel: String,
    pub succeeded: Event,
}

impl Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("model", &self.model)
            .field("event", &self.event)
            .field("filters", &self.filters)
            .field("channel", &self.channel)
            .finish()
    }
}
