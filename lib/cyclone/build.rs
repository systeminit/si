use std::{env, process::Command};

fn main() {
    let cwd = env::current_dir().expect("Unable to obtain current dir");
    let langjs_dir = cwd.join("../../bin/lang-js");

    env::set_current_dir(&langjs_dir).expect("Unable to find bin/lang-js folder");
    let output = Command::new("npm")
        .args(["install"])
        .output()
        .expect("Unable to npm install lang-js");
    if !output.status.success() {
        panic!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new("npm")
        .args(["run", "package"])
        .output()
        .expect("Unable to package lang-js");
    if !output.status.success() {
        panic!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    env::set_current_dir(cwd).expect("Unable to go back to cwd");

    println!("cargo:rerun-if-changed={}", langjs_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
}
