//! Build script for `couchbase-sys` to bind to `libcouchbase`.

use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut build_cfg = cmake::Config::new("libcouchbase");

    if env::var("PROFILE").unwrap() == "release" {
        build_cfg.define("CMAKE_BUILD_TYPE", "RelWithDebInfo");
    } else {
        build_cfg.define("CMAKE_BUILD_TYPE", "DEBUG");
    }

    build_cfg.define("LCB_NO_TESTS", "ON");
    build_cfg.define("LCB_NO_TOOLS", "ON");
    build_cfg.define("LCB_NO_MOCK", "ON");
    build_cfg.define("LCB_BUILD_LIBEV", "OFF");
    build_cfg.define("LCB_BUILD_LIBUV", "OFF");

    let build_dst = build_cfg.build();

    println!(
        "cargo:rustc-link-search=native={}",
        build_dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=couchbase");

    let bindings = bindgen::Builder::default()
        .header("headers.h")
        .clang_arg("-I")
        .clang_arg(format!("{}/include", env::var("OUT_DIR").unwrap()))
        .blacklist_type("max_align_t")
        .generate_comments(false)
        .generate()
        .expect("Unable to generate bindings!");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings!");
}
