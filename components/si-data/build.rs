use std::path::Path;

const PROTOS: &[&str] = &["si-registry/proto/si.data.proto"];

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    for proto in PROTOS {
        println!("cargo:rerun-if-changed=../{}", proto);
    }

    let proto_include_path = Path::new("..").canonicalize().unwrap();
    let proto_include_path_string = proto_include_path.to_str().unwrap();

    let mut prost_build = prost_build::Config::new();
    prost_build
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile_protos(
            &["si-registry/proto/si.data.proto"],
            &[proto_include_path_string],
        )
        .expect("cannot compile protobufs");
}
