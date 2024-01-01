#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

pub enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closed = 2,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub data: String,
}

pub trait Subscribable {
    fn new(url: &str) -> Result<EventSource, String>;
    fn state(&self) -> ReadyState;
    fn close(&mut self);
    fn close_and_notify(&mut self);
    fn subscribe(
        &mut self,
        event_type: impl Into<String>,
        callback: impl Fn(Event) -> () + 'static + Send + Sync,
    ) -> Result<usize, String>;
    fn unsubscribe(&mut self, id: usize) -> Result<(), String>;
}
