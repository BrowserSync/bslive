#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use std::time::Duration;
use tokio::time::sleep;

#[napi]
async fn start(args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
  println!("{:?}", args);
  sleep(Duration::from_secs(2)).await;
  Ok(32)
}
