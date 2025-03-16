use std::process::{Command, Stdio};

#[actix_rt::main]
async fn main() {
    let output = Command::new("npm")
        .arg("prune")
        .env("TERM", "xterm-256color") // Set terminal type
        .env("CLICOLOR_FORCE", "1") // Force colors in many Unix tools
        .env("CLICOLOR", "1") // Enable colors
        .env("COLORTERM", "truecolor") // Indicate full color support
        .spawn();
    // println!("stderr of ls: {:?}", output);
}
