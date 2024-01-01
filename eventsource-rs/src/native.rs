use std::collections::HashMap;

pub struct EventSource {
    es: sse_client::EventSource,
    id_counter: usize,
    listeners: HashMap<usize, usize>,
}

impl crate::Subscribable for EventSource {
    fn new(url: &str) -> Result<Self, String> {
        let es = sse_client::EventSource::new(url).map_err(|e| e.to_string())?;

        Ok(EventSource {
            es,
            id_counter: 0,
            listeners: HashMap::new(),
        })
    }

    fn state(&self) -> crate::ReadyState {
        match self.es.state() {
            sse_client::State::Connecting => crate::ReadyState::Connecting,
            sse_client::State::Open => crate::ReadyState::Open,
            sse_client::State::Closed => crate::ReadyState::Closed,
        }
    }

    fn close(&mut self) {
        self.es.close();
    }

    fn close_and_notify(&mut self) {
        self.es.close();
    }

    fn subscribe(
        &mut self,
        event_type: impl Into<String>,
        callback: impl Fn(crate::Event) -> () + 'static + Send + Sync,
    ) -> Result<usize, String> {
        let topic: String = event_type.into();
        self.es.add_event_listener(topic.as_str(), move |e| {
            let event = crate::Event {
                event_type: e.type_,
                data: e.data,
            };
            callback(event);
        });

        let id = self.id_counter;
        self.id_counter += 1;

        Ok(id)
    }

    fn unsubscribe(&mut self, id: usize) -> Result<(), String> {
        self.listeners
            .remove(&id)
            .ok_or("Listener not found".to_string())
            .map(|_| ())
    }
}

impl Drop for EventSource {
    fn drop(&mut self) {
        use crate::Subscribable;

        self.close_and_notify();
    }
}
