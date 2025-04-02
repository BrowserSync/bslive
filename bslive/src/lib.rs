#![deny(clippy::all)]
#![allow(unexpected_cfgs)]

#[macro_use]
extern crate napi_derive;

use std::time::Duration;

use bsnext_system::cli::from_args;
use tokio::time::sleep;

#[allow(dead_code)]
#[napi]
async fn start(_args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
    eprintln!("async not supported yet");
    sleep(Duration::from_secs(2)).await;
    Ok(32)
}

/// Launch in a blocking way
#[allow(dead_code)]
#[napi]
fn start_sync(args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
    let sys = actix_rt::System::new();
    let result = sys.block_on(async move {
        match from_args(args).await {
            Ok(_) => 0,
            Err(_) => 1,
        }
    });
    Ok(result)
}
