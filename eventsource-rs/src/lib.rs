// #[cfg(not(target_arch = "wasm32"))]
pub mod native;

// #[cfg(not(target_arch = "wasm32"))]
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

pub struct Event {
    event_type: String,
    data: String,
}

pub trait EventSource {
    fn new(url: &str) -> Result<impl EventSource, String>;
    fn state(&self) -> ReadyState;
    fn close(&mut self);
    fn close_and_notify(&mut self);
    fn add_event_listener(
        &mut self,
        event_type: impl Into<String>,
        callback: impl Fn(Event) -> () + 'static,
    ) -> Result<usize, String>;
    fn remove_event_listener(&mut self, id: usize) -> Result<(), String>;
}
