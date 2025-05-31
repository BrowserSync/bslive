#![deny(clippy::all)]
#![allow(unexpected_cfgs)]

#[macro_use]
extern crate napi_derive;

use bsnext_system::cli::from_args;

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

/// Launch in a none-blocking way
#[allow(dead_code)]
#[napi]
fn start(args: Vec<String>) -> () {
    println!("{args:?}");
    ()
}
