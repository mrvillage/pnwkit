use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{sync::Arc, time::Duration};

use pnwkit_core::{async_trait, json, json_from_str, DashMap, Event, Subscription, Value};
use tokio::sync::{Mutex, RwLock};
use tokio::{net::TcpStream, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct Socket {
    kit: Mutex<Option<&'static pnwkit_core::Kit>>,
    established: Event,
    connected: Event,
    socket_id: RwLock<Option<String>>,
    activity_timeout: RwLock<u16>,
    subscriptions: Arc<RwLock<DashMap<String, Arc<Subscription>>>>,
    ws: Mutex<Option<SplitSink<WsStream, Message>>>,
    last_message: RwLock<Option<Instant>>,
    ponged: Mutex<bool>,
    pinged: Mutex<bool>,
}

impl Socket {
    pub fn new() -> Self {
        Socket {
            kit: Mutex::new(None),
            established: Event::new(),
            connected: Event::new(),
            socket_id: RwLock::new(None),
            activity_timeout: RwLock::new(120),
            subscriptions: Arc::new(RwLock::new(DashMap::new())),
            ws: Mutex::new(None),
            last_message: RwLock::new(None),
            ponged: Mutex::new(false),
            pinged: Mutex::new(false),
        }
    }
}

#[async_trait]
impl pnwkit_core::Socket for Socket {
    async fn init(&self, kit: &'static pnwkit_core::Kit) {
        let mut l = self.kit.lock().await;
        if l.is_none() {
            l.replace(kit);
        }
    }

    fn get_established(&'_ self) -> &'_ Event {
        &self.established
    }

    fn get_connected(&'_ self) -> &'_ Event {
        &self.connected
    }

    async fn get_socket_id(&self) -> String {
        self.socket_id.read().await.as_ref().unwrap().clone()
    }

    async fn add_subscription(&self, subscription: Arc<Subscription>) {
        let channel = subscription.channel.lock().await.clone();
        self.subscriptions
            .write()
            .await
            .insert(channel, subscription);
    }

    async fn remove_subscription(&self, subscription: Arc<Subscription>) {
        self.subscriptions
            .write()
            .await
            .remove(subscription.channel.lock().await.as_str());
    }

    async fn get_subscription(&self, channel: String) -> Option<Arc<Subscription>> {
        self.subscriptions
            .read()
            .await
            .get(&channel)
            .map(|v| v.value().clone())
    }

    async fn send(&self, data: String) -> Result<(), String> {
        self.ws
            .lock()
            .await
            .as_mut()
            .unwrap()
            .send(Message::Text(data))
            .await
            .map_err(|e| e.to_string())
    }

    async fn connect(&'static self, url: &str) -> Result<(), String> {
        self.connected.set().await;
        let res: Result<(WsStream, _), _> = connect_async(url).await;
        if let Err(err) = res {
            self.connected.clear().await;
            return Err(err.to_string());
        }
        let (ws, _) = res.unwrap();
        let (write, read) = ws.split();
        self.ws.lock().await.replace(write);
        tokio::spawn(read.for_each(move |msg| async {
            tokio::spawn(async {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Text(text) => {
                            {
                                *self.last_message.write().await = Some(Instant::now());
                            }
                            let ws_event =
                                json_from_str::<Value>(&text).unwrap().as_object().unwrap();
                            let event = ws_event.get("event").unwrap();
                            let event = event.as_str().unwrap();
                            match event {
                                "pusher:connection_established" => {
                                    let data = ws_event.get("data").unwrap().as_object().unwrap();
                                    let socket_id =
                                        data.get("socket_id").unwrap().as_string().unwrap();
                                    let activity_timeout =
                                        data.get("activity_timeout").unwrap().as_i32().unwrap()
                                            as u16;
                                    {
                                        self.socket_id.write().await.replace(socket_id);
                                    }
                                    {
                                        let mut current = self.activity_timeout.write().await;
                                        if *current > activity_timeout {
                                            *current = activity_timeout;
                                        }
                                    }
                                    self.established.set().await;
                                },
                                "pusher_internal:subscription_succeeded" => {
                                    let channel =
                                        ws_event.get("channel").unwrap().as_string().unwrap();
                                    let subscription = self.get_subscription(channel).await;
                                    if let Some(subscription) = subscription {
                                        subscription.succeeded.set().await;
                                    }
                                },
                                "pusher:pong" => {
                                    *self.ponged.lock().await = true;
                                    *self.pinged.lock().await = false;
                                },
                                "pusher:ping" => {
                                    if self
                                        .send(
                                            json!({"event": "pusher:pong", "data": {}}).to_string(),
                                        )
                                        .await
                                        .is_err()
                                    {}
                                },
                                _ => {
                                    let data = ws_event.get("data").unwrap();
                                    let channel = ws_event
                                        .get("channel")
                                        .unwrap()
                                        .value()
                                        .as_string()
                                        .unwrap();
                                    if let Some(subscription) = self.get_subscription(channel).await
                                    {
                                        let callbacks = subscription.callbacks.read().await;
                                        if event.starts_with("BULK_") {
                                            let data = data.as_array().unwrap();
                                            for item in data {
                                                let item = item.as_object().unwrap();
                                                for callback in callbacks.iter() {
                                                    tokio::spawn(callback(&item));
                                                }
                                            }
                                        } else {
                                            let data = data.as_object().unwrap();
                                            for callback in callbacks.iter() {
                                                tokio::spawn(callback(&data));
                                            }
                                        }
                                    }
                                },
                            }
                        },
                        Message::Ping(_) => {
                            if self
                                .ws
                                .lock()
                                .await
                                .as_mut()
                                .unwrap()
                                .send(Message::Pong(Vec::new()))
                                .await
                                .is_err()
                            {}
                        },
                        Message::Close(frame) => {
                            self.established.clear().await;
                            self.ws.lock().await.take();
                            if let Some(f) = frame {
                                let code: u16 = f.code.into();
                                if (4000..4100).contains(&code) {
                                    panic!("socket closed with code {}", code);
                                } else if (4100..4200).contains(&code) {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    let res = self.reconnect().await;
                                    if let Err(err) = res {
                                        panic!("reconnect failed: {}", err);
                                    }
                                } else {
                                    let res = self.reconnect().await;
                                    if let Err(err) = res {
                                        panic!("reconnect failed: {}", err);
                                    }
                                }
                            } else {
                                panic!("socket closed without code");
                            }
                        },
                        _ => {},
                    }
                }
            });
        }));
        Ok(())
    }

    async fn reconnect(&'static self) -> Result<(), String> {
        let res = self
            .connect(&self.kit.lock().await.unwrap().config.socket_url)
            .await;
        if let Err(err) = res {
            return Err(err);
        }
        {
            *self.ponged.lock().await = true;
            *self.pinged.lock().await = false;
        }
        for s in self.subscriptions.read().await.iter() {
            let (_, subscription) = s.pair();
            subscription.succeeded.clear().await;
            // resubscribe
        }
        Ok(())
    }

    async fn ping_pong(&'static self) {
        loop {
            let last_message = { *self.last_message.read().await };
            let activity_timeout = { *self.activity_timeout.read().await };
            if last_message.is_none() {
                tokio::time::sleep(Duration::from_secs(activity_timeout.into())).await;
                continue;
            }
            let elapsed = last_message.unwrap().elapsed();
            if elapsed.as_secs() >= activity_timeout.into() {
                let pinged = { *self.pinged.lock().await };
                if pinged {
                    // if pinged, wait a bit so it doesn't go into an infinite loop
                    tokio::time::sleep(Duration::from_secs(2)).await;
                } else {
                    let res = self
                        .send(json!({"event": "pusher:ping", "data": {}}).to_string())
                        .await;
                    if res.is_err() {
                        // if err, wait a bit to give the issue time to fix itself then continue
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    {
                        *self.pinged.lock().await = true;
                        *self.ponged.lock().await = false;
                    }
                    tokio::spawn(self.call_later_pong());
                }
            }
        }
    }

    async fn call_later_pong(&'static self) {
        tokio::time::sleep(Duration::from_secs(30)).await;
        if !*self.ponged.lock().await {
            self.established.clear().await;
            self.ws.lock().await.take();
            if let Err(err) = self.reconnect().await {
                panic!("timed out waiting for ping, reconnect failed: {}", err);
            }
        }
    }
}
