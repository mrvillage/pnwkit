use serde_json::json;
#[cfg(feature = "subscriptions")]
use std::{sync::Arc, time::Duration};

use crate::{
    data::QueryReturn,
    query::{Query, QueryType},
    request::{ContentType, Method, Request, Response},
    resolve::Resolve,
    variable::Variables,
    Config, Data, Field, Paginator,
};
#[cfg(feature = "subscriptions")]
use crate::{
    data::SubscriptionAuthData,
    subscription::{Subscription, SubscriptionEvent, SubscriptionModel},
    to_query_string::ToQueryString,
    Object,
};

type GetResult = Result<Data, String>;

#[cfg(feature = "subscriptions")]
type SubscriptionResult = Result<Arc<Subscription>, String>;

#[derive(Debug)]
pub struct Kit {
    pub config: Config,
}

impl Kit {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    #[cfg(feature = "async")]
    pub async fn get(&self, query: &Query) -> GetResult {
        self.inner_get(query, None).await
    }

    #[cfg(feature = "sync")]
    pub fn get_sync(&self, query: &Query) -> GetResult {
        self.inner_get_sync(query, None)
    }

    #[cfg(feature = "async")]
    pub async fn get_with_variables(&self, query: &Query, variables: &Variables) -> GetResult {
        self.inner_get(query, Some(variables)).await
    }

    #[cfg(feature = "sync")]
    pub fn get_with_variables_sync(&self, query: &Query, variables: &Variables) -> GetResult {
        self.inner_get_sync(query, Some(variables))
    }

    fn parse_response(&self, response: Response) -> GetResult {
        let result = serde_json::from_str::<QueryReturn>(&response.body);
        match result {
            Ok(json) => {
                if json.errors.is_some() {
                    let errors = json.errors.unwrap();
                    let mut error_messages = Vec::with_capacity(errors.len());
                    for error in errors {
                        error_messages.push(error.message);
                    }
                    return Err(error_messages.join(", "));
                }
                match json.data {
                    Some(d) => Ok(d),
                    None => Err("No data".to_string()),
                }
            },
            Err(err) => Err(err.to_string()),
        }
    }

    // this should work fine without the drop, but it's here just in case
    fn hit(&self) -> u64 {
        let mut rate_limiter = self.config.rate_limiter.lock().unwrap();
        let wait = rate_limiter.hit();
        drop(rate_limiter);
        wait
    }

    fn handle_429(&self, x_ratelimit_reset: Option<u64>) -> u64 {
        let mut rate_limiter = self.config.rate_limiter.lock().unwrap();
        let wait = rate_limiter.handle_429(x_ratelimit_reset);
        drop(rate_limiter);
        wait
    }

    #[cfg(feature = "async")]
    async fn inner_get(&self, query: &Query, variables: Option<&Variables>) -> GetResult {
        let request = self.build_request(query, variables);
        if let Err(msg) = &request {
            return Err(msg.clone());
        }
        let request = request.unwrap();
        let mut err_msg = "Something went very wrong".to_string();
        for _ in 1..5 {
            loop {
                let wait = self.hit();
                if wait > 0 {
                    (self.config.sleep)(Duration::from_secs(wait)).await;
                } else {
                    break;
                }
            }
            let response = self.config.client.request(&request).await;
            if let Err(err) = response {
                err_msg = err.to_string();
                continue;
            }
            let response = response.unwrap();
            if response.status == 429 {
                let wait = self.handle_429(response.x_ratelimit_reset);
                (self.config.sleep)(Duration::from_secs(wait)).await;
            }
            return self.parse_response(response);
        }
        Err(format!("Max retries exceeded, returned error: {}", err_msg))
    }

    #[cfg(feature = "sync")]
    fn inner_get_sync(&self, query: &Query, variables: Option<&Variables>) -> GetResult {
        let request = self.build_request(query, variables);
        if let Err(msg) = &request {
            return Err(msg.clone());
        }
        let request = request.unwrap();
        let mut err_msg = "Something went very wrong".to_string();
        for _ in 1..5 {
            loop {
                let wait = self.hit();
                if wait > 0 {
                    (self.config.sleep_sync)(Duration::from_secs(wait));
                } else {
                    break;
                }
            }
            let response = self.config.client.request_sync(&request);
            if let Err(err) = response {
                err_msg = err.to_string();
                continue;
            }
            let response = response.unwrap();
            if response.status == 429 {
                let wait = self.handle_429(response.x_ratelimit_reset);
                (self.config.sleep_sync)(Duration::from_secs(wait));
            }
            return self.parse_response(response);
        }
        Err(format!("Max retries exceeded, returned error: {}", err_msg))
    }

    pub fn build_request(
        &self,
        query: &Query,
        variables: Option<&Variables>,
    ) -> Result<Request, String> {
        if let Err(msg) = query.valid() {
            return Err(format!("Invalid query: {}", msg));
        }
        if let Some(v) = variables {
            if let Err(msg) = v.valid(
                query
                    .get_variables()
                    .iter()
                    .map(|v| v.name.clone())
                    .collect(),
            ) {
                return Err(format!("Invalid variables: {}", msg));
            }
        }
        let body = match variables {
            Some(vars) => json!({
                "query": query.resolve(),
                "variables": vars,
            }),
            None => json!({
                "query": query.resolve(),
            }),
        };
        let method = Method::Get;
        Ok(Request::new(
            method,
            self.config.api_url.clone(),
            Some(serde_json::to_string(&body).unwrap()),
            Some(self.config.headers.clone()),
            Some(ContentType::Json),
        ))
    }

    pub fn query(&self) -> Query {
        Query::new(QueryType::Query)
    }

    pub fn mutation(&self) -> Query {
        Query::new(QueryType::Mutation)
    }

    pub fn paginator(&self, field: Field) -> Paginator {
        let query = Query::new(QueryType::Query).field(field);
        Paginator::new(query)
    }

    pub fn paginator_with_capacity(&self, field: Field, capacity: u16) -> Paginator {
        let query = Query::new(QueryType::Query).field(field);
        Paginator::with_capacity(query, capacity)
    }

    pub fn paginator_with_variables(&self, field: Field, variables: Variables) -> Paginator {
        let query = Query::new(QueryType::Query).field(field);
        Paginator::with_variables(query, variables)
    }

    pub fn paginator_with_capacity_and_variables(
        &self,
        field: Field,
        variables: Variables,
        capacity: u16,
    ) -> Paginator {
        let query = Query::new(QueryType::Query).field(field);
        Paginator::with_capacity_and_variables(query, capacity, variables)
    }

    #[cfg(feature = "subscriptions")]
    pub async fn subscribe(
        &'static self,
        model: SubscriptionModel,
        event: SubscriptionEvent,
    ) -> SubscriptionResult {
        self.subscribe_inner(model, event, Object::new()).await
    }

    #[cfg(feature = "subscriptions")]
    pub async fn subscribe_with_filters(
        &'static self,
        model: SubscriptionModel,
        event: SubscriptionEvent,
        filters: Object,
    ) -> SubscriptionResult {
        self.subscribe_inner(model, event, filters).await
    }

    #[cfg(feature = "subscriptions")]
    async fn subscribe_inner(
        &'static self,
        model: SubscriptionModel,
        event: SubscriptionEvent,
        filters: Object,
    ) -> SubscriptionResult {
        self.config.socket.init(self).await;
        let channel = self
            .request_subscription_channel(&model, &event, &filters)
            .await;
        if let Err(e) = channel {
            return Err(e);
        }

        let subscription = Subscription::new(model, event, filters, channel.unwrap());

        self.subscribe_request(subscription).await
    }

    #[cfg(feature = "subscriptions")]
    async fn subscribe_request(&'static self, subscription: Subscription) -> SubscriptionResult {
        if self.config.socket.get_connected().is_set().await {
            let res = self.config.socket.connect(&self.config.socket_url).await;
            if let Err(e) = res {
                return Err(e);
            }
        }

        let mut channel = { subscription.channel.lock().await.clone() };
        let auth = self.authorize_subscription(&channel).await;
        if let Err(e) = &auth {
            if e == "unauthorized" {
                let res = self
                    .request_subscription_channel(
                        &subscription.model,
                        &subscription.event,
                        &subscription.filters,
                    )
                    .await;
                if let Err(e) = res {
                    return Err(e);
                }
                channel = res.unwrap();
                subscription.set_channel(channel.clone()).await;
                let auth = self.authorize_subscription(&channel).await;
                if let Err(e) = &auth {
                    return Err(e.clone());
                }
            }
        }
        let auth = auth.unwrap();

        let subscription = Arc::new(subscription);
        self.config
            .socket
            .add_subscription(subscription.clone())
            .await;

        let send = self.config.socket.send(
            json!({
                "event": "pusher:subscribe",
                "data": {
                    "channel": channel,
                    "auth": auth.clone(),
                }
            })
            .to_string(),
        );

        if let Err(e) = send.await {
            return Err(e);
        }

        let timeout =
            tokio::time::timeout(Duration::from_secs(60), subscription.succeeded.wait()).await;
        if timeout.is_err() {
            self.config
                .socket
                .remove_subscription(subscription.clone())
                .await;
            return Err("timed out waiting for subscription to succeed".to_string());
        }

        Ok(subscription.clone())
    }

    #[cfg(feature = "subscriptions")]
    async fn request_subscription_channel(
        &self,
        model: &SubscriptionModel,
        event: &SubscriptionEvent,
        filters: &Object,
    ) -> Result<String, String> {
        let url = self
            .config
            .subscribe_url
            .replace("{model}", &model.to_string())
            .replace("{event}", &event.to_string());
        let url = if !filters.is_empty() {
            format!(
                "{}?{}",
                url,
                serde_urlencoded::to_string(filters.to_query_string()).unwrap()
            )
        } else {
            url
        };
        let request = Request::new(
            Method::Get,
            url,
            None,
            Some(self.config.headers.clone()),
            Some(ContentType::Json),
        );
        let response = self.config.client.request(&request).await;
        if let Err(err) = response {
            Err(err)
        } else {
            let response = response.unwrap();
            let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
            if let Some(err) = json.get("error") {
                return Err(err.to_string());
            }
            if let Some(channel) = json.get("channel") {
                return Ok(channel.to_string());
            }
            Err("malformed response".to_string())
        }
    }

    #[cfg(feature = "subscriptions")]
    async fn authorize_subscription(&self, channel: &String) -> Result<String, String> {
        self.config.socket.get_established().wait().await;
        let request = Request::new(
            Method::Post,
            self.config.subscription_auth_url.clone(),
            Some(
                serde_urlencoded::to_string([
                    ("socket_id", self.config.socket.get_socket_id().await),
                    ("channel_name", channel.to_owned()),
                ])
                .unwrap(),
            ),
            None,
            Some(ContentType::Form),
        );
        let response = self.config.client.request(&request).await;
        if let Err(e) = response {
            return Err(e);
        }
        let response = response.unwrap();
        if response.status != 200 {
            return Err("unauthorized".into());
        }
        let data = serde_json::from_str::<SubscriptionAuthData>(&response.body).unwrap();
        Ok(data.auth)
    }
}
