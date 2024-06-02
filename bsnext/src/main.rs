use actix_rt::System;
use std::env::args;
use std::process;

use bsnext_system::cli::from_args;

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli_args = args();
    let code = match from_args(cli_args).await {
        Ok(_) => 0,
        Err(_) => 1,
    };
    System::current().stop_with_code(code);
    process::exit(code)
}
