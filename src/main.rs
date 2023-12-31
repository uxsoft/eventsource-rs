use std::collections::HashMap;

use serde::Deserialize;
use spinta::EsEvent;
use ureq::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectMessage {
    client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Action {
    Update,
    Create,
    Delete,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordChangedMessage<T> {
    record: T,
    action: Action,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Post {
    collection_id: String,
    collection_name: String,
    created: String,
    id: String,
    title: String,
    updated: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordBase {
    collection_id: String,
    collection_name: String,
    created: String,
    id: String,
    updated: String,
}

const ENDPOINT: &str = "http://localhost:8090/api/realtime";

struct Subscription {
    topic: String,
    callback: fn(String) -> (),
}

struct RealtimeManager {
    subscriptions: Vec<Subscription>,
    client_id: Option<String>,
}

impl RealtimeManager {
    fn post_subscriptions(&self) -> bool {
        println!("Posting topics");

        let topics: Vec<_> = self
            .subscriptions
            .iter()
            .map(|s| s.topic.as_str())
            .collect();

        if topics.len() == 0 || self.client_id.is_none() {
            return false;
        }

        println!("Posting topics: {:?}", topics);

        ureq::post(ENDPOINT)
            .send_json(ureq::json!({
                "clientId": self.client_id,
                "subscriptions": topics,
            }))
            .is_ok()
    }

    fn notify(&self, topic: String, payload: String) {
        self.subscriptions
            .iter()
            .filter(|s| s.topic == topic)
            .for_each(|s| (s.callback)(payload.clone()))
    }

    pub fn subscribe<S: Into<String>>(&mut self, topic: S, callback: fn(String) -> ()) {
        self.subscriptions.push(Subscription {
            topic: topic.into(),
            callback,
        });

        match self.client_id {
            Some(_) => {
                self.post_subscriptions();
            }
            None => {
                self.connect();
            }
        };
    }

    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
            client_id: None,
        }
    }

    fn on_message(&mut self, event: EsEvent) {
        match event {
            spinta::EsEvent::Opened => {
                println!("Opened");
            }
            spinta::EsEvent::Message(payload) => {
                println!("Message `{}`", payload);

                let message_opt = serde_json::from_str::<ConnectMessage>(payload.as_str());
                if let Ok(message) = message_opt {
                    println!("Client Id: {}", message.client_id);
                    self.client_id = Some(message.client_id);
                    self.post_subscriptions();
                }

                let change_opt =
                    serde_json::from_str::<RecordChangedMessage<RecordBase>>(payload.as_str());
                if let Ok(change) = change_opt {
                    println!("Change: {:?}", change);
                    self.notify(change.record.collection_name, payload);
                }
            }
            spinta::EsEvent::Error(e) => {
                eprintln!("Error: `{}`", e);
            }
            spinta::EsEvent::Closed => {
                println!("Closed");
            }
        }
    }

    pub fn connect(&mut self) {
        let receiver =
            spinta::connect(ENDPOINT).expect("Failed to connect to the server over SSE.");

        let receiver = spinta::connect_with_wakeup(ENDPOINT, || {
            println!("Houston, we have an event!");
            while let Some(event) = receiver.try_recv() {
                self.on_message(event);
            }
        });

    }
}

#[tokio::main]
async fn main() {
    let mut realtime = RealtimeManager::new();

    realtime.connect();

    realtime.subscribe("posts", |payload| {
        println!("Post! {}", payload);
    });

    realtime.subscribe("users", |payload| {
        println!("Post! {}", payload);
    });
}
