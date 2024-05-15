use std::env::current_dir;
use std::{env, io};
use std::io::Write;
use std::process::Command;

fn main() {
    if env::var("CI").is_ok() {
        return;
    }
    let curr = current_dir().expect("current dir");
    let root = curr
        .parent()
        .expect("parent")
        .parent()
        .expect("parent 2")
        .to_path_buf();
    let output = Command::new("typeshare")
        .args([
            "crates/bsnext_core",
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
        .args(["run", "build"])
        .current_dir(root.join("crates").join("bsnext_client"))
        .output()
        .expect("sh command failed to start");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());
}
