use js_sys::wasm_bindgen::{closure::Closure, JsCast, JsValue};
use std::collections::HashMap;
use web_sys::MessageEvent;

pub struct EventSource {
    es: web_sys::EventSource,
    listeners: HashMap<usize, js_sys::Function>,
    event_types: HashMap<usize, String>,
    id_counter: usize,
}

impl crate::Subscribable for EventSource {
    fn new(url: &str) -> Result<Self, String> {
        let es = web_sys::EventSource::new(url).map_err(js_value_to_string)?;

        Ok(Self {
            es,
            listeners: HashMap::new(),
            event_types: HashMap::new(),
            id_counter: 0,
        })
    }

    fn state(&self) -> crate::ReadyState {
        match self.es.ready_state() {
            0 => crate::ReadyState::Connecting,
            1 => crate::ReadyState::Open,
            2 => crate::ReadyState::Closed,
            _ => unreachable!(),
        }
    }

    fn subscribe(
        &mut self,
        event_type: impl Into<String>,
        callback: impl Fn(crate::Event) -> () + 'static,
    ) -> Result<usize, String> {
        let topic: String = event_type.into();
        
        let closure_topic = topic.clone();
        let closure = Closure::wrap(Box::new(move |e: MessageEvent| {
            let event_data = e.data().as_string().unwrap().into();
            let event = crate::Event {
                event_type: closure_topic.clone(),
                data: event_data,
            };
            callback(event);
        }) as Box<dyn FnMut(MessageEvent)>);

        let jsfn_ref: &js_sys::Function = closure.as_ref().unchecked_ref();
        let jsfn = jsfn_ref.clone();

        let id = self.id_counter;
        self.listeners.insert(id, jsfn);
        self.event_types.insert(id, topic.clone());
        self.id_counter += 1;

        let result = self
            .es
            .add_event_listener_with_callback(&topic, self.listeners.get(&id).unwrap())
            .map_err(js_value_to_string);

        result.map(|_| id)
    }

    fn unsubscribe(&mut self, id: usize) -> Result<(), String> {
        let callback = self.listeners.get(&id).unwrap();
        let event_type = self.event_types.get(&id).unwrap();

        self.es
            .remove_event_listener_with_callback(event_type, callback)
            .map_err(js_value_to_string)
    }

    fn close(&mut self) {
        self.es.close();
    }

    fn close_and_notify(&mut self) {
        self.es.close();
        // Fire an error event to cause all subscriber
        // streams to close down.
        if let Ok(event) = web_sys::Event::new("error") {
            let _ = self.es.dispatch_event(&event);
        }
    }
}

fn js_value_to_string(item: JsValue) -> String {
    js_sys::JSON::stringify(&item).unwrap().into()
}

impl Drop for EventSource {
    fn drop(&mut self) {
        use crate::Subscribable;

        self.close_and_notify();
    }
}
