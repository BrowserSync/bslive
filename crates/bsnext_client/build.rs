use std::env::current_dir;
use std::io::Write;
use std::process::Command;
use std::{env, io};

fn main() {
    if env::var("CI").is_ok() {
        return;
    }
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=../bsnext_dto");
    let curr = current_dir().expect("current dir");
    let root = curr
        .parent()
        .expect("parent")
        .parent()
        .expect("parent 2")
        .to_path_buf();
    let output = Command::new("typeshare")
        .args([
            "crates/bsnext_dto",
            "--lang=typescript",
            "--output-file=crates/bsnext_client/generated/dto.ts",
        ])
        .current_dir(&root)
        .output()
        .expect("build types");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let output = Command::new("npm")
        .args(["run", "schema"])
        .current_dir(root.join("crates").join("bsnext_client"))
        .output()
        .expect("build schema");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let output = Command::new("npm")
        .args(["run", "build:client"])
        .current_dir(root)
        .output()
        .expect("sh command failed to start");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());
}
