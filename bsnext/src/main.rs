use actix_rt::System;
use std::env::args;
use std::process;

use bsnext_system::cli::from_args;

fn main() {
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
    match from_args(cli_args).await {
        Ok(_) => 0,
        Err(_) => 1,
    }
}
