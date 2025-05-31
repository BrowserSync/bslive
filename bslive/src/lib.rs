#![deny(clippy::all)]
#![allow(unexpected_cfgs)]

#[macro_use]
extern crate napi_derive;

use bsnext_system::cli::from_args;
use napi::bindgen_prelude::{AbortSignal, AsyncTask};
use napi::{Env, JsNumber};

/// Launch in a blocking way
#[allow(dead_code)]
#[napi]
fn start_blocking(args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
    let sys = actix_rt::System::new();
    let result = sys.block_on(async move {
        match from_args(args).await {
            Ok(_) => 0,
            Err(_) => 1,
        }
    });
    Ok(result)
}

pub struct AsyncStart {
    args: Vec<String>,
    rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl napi::Task for AsyncStart {
    type Output = i32;
    type JsValue = JsNumber;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let sys = actix_rt::System::new();
        let args = self.args.clone();
        let rx = self.rx.take().expect("must be there");
        let result = sys.block_on(async move {
            tokio::select! {
                _ = rx => {
                    println!("did exit from one-shot");
                    2
                }
                res = from_args(args) => {
                    println!("did exit from server-shot");
                    match res {
                        Ok(_) => 0,
                        Err(_) => 1,
                    }
                }
            }
        });
        Ok(result)
    }

    fn resolve(&mut self, env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(env.create_int32(output)?)
    }
}

#[napi(js_name = "BsSystem")]
pub struct JsBsSystem {
    system: BsSystem,
}

pub struct BsSystem {
    sender: Option<tokio::sync::oneshot::Sender<()>>,
}

impl BsSystem {
    pub fn new() -> Self {
        Self { sender: None }
    }
}

#[napi]
impl JsBsSystem {
    #[napi(constructor)]
    pub fn new() -> Self {
        JsBsSystem {
            system: BsSystem::new(),
        }
    }
    #[napi]
    pub fn start(&mut self, args: Vec<String>, signal: AbortSignal) -> AsyncTask<AsyncStart> {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        self.system.sender = Some(tx);
        AsyncTask::with_signal(AsyncStart { args, rx: Some(rx) }, signal)
    }
    #[napi]
    pub fn send(&self, arg: serde_json::Value) -> () {
        println!("try to send? {arg:?}");
    }
    #[napi]
    pub fn stop(&mut self) -> () {
        if let Some(sender) = self.system.sender.take() {
            sender.send(()).unwrap()
        }
    }
}
