#![deny(clippy::all)]
#![allow(unexpected_cfgs)]

#[macro_use]
extern crate napi_derive;

use crate::async_start::AsyncStart;
use napi::bindgen_prelude::{AbortSignal, AsyncTask};

mod async_start;
mod blocking_start;

#[napi(js_name = "BsSystem")]
pub struct JsBsSystem {
    system: BsSystem,
}

pub struct BsSystem {
    sender: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Default for BsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl BsSystem {
    pub fn new() -> Self {
        Self { sender: None }
    }
}

#[napi]
impl JsBsSystem {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        JsBsSystem {
            system: BsSystem::new(),
        }
    }
    #[napi(ts_return_type = "Promise<number>")]
    pub fn start(&mut self, args: Vec<String>, signal: AbortSignal) -> AsyncTask<AsyncStart> {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        self.system.sender = Some(tx);
        AsyncTask::with_signal(AsyncStart { args, rx: Some(rx) }, signal)
    }
    #[napi]
    pub fn send(&self, arg: serde_json::Value) {
        println!("try to send? {arg:?}");
    }
    #[napi]
    pub fn stop(&mut self) {
        if let Some(sender) = self.system.sender.take() {
            sender.send(()).unwrap()
        }
    }
}
