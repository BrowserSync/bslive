use actix_rt::System;
use std::env::{args, current_dir};
use std::path::PathBuf;
use std::process;

use bsnext_system::cli::from_args;

fn main() {
    unsafe {
        std::env::set_var("RUST_LIB_BACKTRACE", "0");
    }
    let code = System::with_tokio_rt(|| {
        // build system with a multi-thread tokio runtime.
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    })
    .block_on(async_main());
    System::current().stop_with_code(code);
    process::exit(code)
}

async fn async_main() -> i32 {
    let cli_args = args();
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    unsafe {
        std::env::set_var("RUST_LIB_BACKTRACE", "0");
    }
    match from_args(cli_args, cwd).await {
        Ok(_) => 0,
        Err(_) => 1,
    }
}
