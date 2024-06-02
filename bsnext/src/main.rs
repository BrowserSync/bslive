use std::env::args;
use std::process;

use bsnext_system::cli::from_args;

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli_args = args();
    process::exit(match from_args(cli_args).await {
        Ok(_) => 0,
        Err(_) => 1,
    })
}
