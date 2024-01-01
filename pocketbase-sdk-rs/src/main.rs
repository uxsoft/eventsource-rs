use std::collections::HashMap;

use eventsource_rs::{Event, EventSource, Subscribable};
use serde::Deserialize;
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

struct Subscription {
    topic: String,
    callback: fn(String) -> (),
}

struct RealtimeManager {
    subscriptions: Vec<Subscription>,
    client_id: Option<String>,
    es: eventsource_rs::EventSource,
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

    pub fn new(url: &str) -> Result<Self, String> {
        let es = EventSource::new(url)?;

        Ok(Self {
            subscriptions: Vec::new(),
            client_id: None,
            es: es as EventSource,
        })
    }

    fn on_message(&mut self, event: Event) {
        println!("Message `{}`", event.data);

        let message_opt = serde_json::from_str::<ConnectMessage>(&event.data);
        if let Ok(message) = message_opt {
            println!("Client Id: {}", message.client_id);
            self.client_id = Some(message.client_id);
            self.post_subscriptions();
        }

        let change_opt = serde_json::from_str::<RecordChangedMessage<RecordBase>>(&event.data);
        if let Ok(change) = change_opt {
            println!("Change: {:?}", change);
            self.notify(change.record.collection_name, event.data);
        }
    }

    pub fn connect(&mut self) {
        // self.es.subscribe("message", move |e| {
        //     self.on_message(e.clone());
        //     println!("{}", e.data)
        // }).unwrap();
    }
}

const ENDPOINT: &str = "http://localhost:8090/api/realtime";

#[tokio::main]
async fn main() {
    let mut realtime = RealtimeManager::new(ENDPOINT).unwrap();

    realtime.connect();

    realtime.subscribe("posts", |payload| {
        println!("Post! {}", payload);
    });

    realtime.subscribe("users", |payload| {
        println!("Post! {}", payload);
    });
}
