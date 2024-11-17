use std::env::current_dir;
use std::io::Write;
use std::process::Command;
use std::{env, io};

#[cfg(not(target_os = "windows"))]
fn main() {
    if env::var("CI").is_ok() {
        return;
    }
    println!("cargo::rerun-if-changed=./src");
    println!("cargo::rerun-if-changed=../../inject/src");
    println!("cargo::rerun-if-changed=../../inject/vendor/live-reload/src");
    println!("cargo::rerun-if-changed=../../ui/src");
    println!("cargo::rerun-if-changed=../../ui/styles");
    println!("cargo::rerun-if-changed=../../ui/svg");

    let curr = current_dir().expect("current dir");
    let root = curr
        .parent()
        .expect("parent")
        .parent()
        .expect("parent 2")
        .to_path_buf();
    let output = Command::new("bash")
        .args(["gen.sh"])
        .current_dir(&root)
        .output()
        .expect("build types");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());
}

#[cfg(target_os = "windows")]
fn main() {}
