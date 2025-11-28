use std::{
    env,
    path::Path,
    process::Command,
};

pub fn generate_git_metadata_rust() {
    #[expect(
        clippy::disallowed_methods,
        reason = "used in Cargo buildscripts, relying on this environment variable"
    )]
    let crate_root = env::var("CARGO_MANIFEST_DIR").expect(
        "Failed to compute Cargo Manifest dir root. This code is only supported running with Cargo",
    );
    let crate_root_path = Path::new(&crate_root);

    #[expect(
        clippy::disallowed_methods,
        reason = "used in Cargo buildscripts, relying on this environment variable"
    )]
    let out_dir = env::var("OUT_DIR").expect(
        "OUT_DIR not set in Cargo build script. This code is only supported running with Cargo",
    );
    let out_dir_path = Path::new(&out_dir);

    let git_metadata_json_path = out_dir_path.join("git_metadata.json");
    let git_metadata_rust_path = out_dir_path.join("git_metadata.rs");

    let generate_git_metadata_script = crate_root_path
        .join("../../prelude-si/build_metadata/generate_git_metadata.py")
        .canonicalize()
        .expect("Failed to canonicalize Buck2 Python script location");
    let generate_git_metadata_rust_script = crate_root_path
        .join("../../prelude-si/build_metadata/generate_git_metadata_rust.py")
        .canonicalize()
        .expect("Failed to canonicalize Buck2 Python script location");

    // Tell Cargo to re-run this build script if the Python scripts change
    println!(
        "cargo:rerun-if-changed={}",
        generate_git_metadata_script.display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        generate_git_metadata_rust_script.display()
    );

    generate_json(&generate_git_metadata_script, &git_metadata_json_path);
    generate_rust(
        &generate_git_metadata_rust_script,
        &git_metadata_json_path,
        &git_metadata_rust_path,
    );
}

fn generate_json(script: &Path, output: &Path) {
    let status = Command::new("python3")
        .arg(script)
        .arg(output)
        .status()
        .expect("Failed to run metadata creation script");
    if !status.success() {
        panic!("Metadata creation script was not successful");
    }
}

fn generate_rust(script: &Path, input: &Path, output: &Path) {
    let status = Command::new("python3")
        .arg(script)
        .arg(input)
        .arg(output)
        .status()
        .expect("Failed to run Rust code creation script");
    if !status.success() {
        panic!("Rust code creation script was not successful");
    }
}
