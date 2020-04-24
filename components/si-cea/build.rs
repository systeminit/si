use std::path::Path;

const PROTOS: &[&str] = &["si-registry/proto/si.cea.proto"];

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    for proto in PROTOS {
        println!("cargo:rerun-if-changed=../{}", proto);
    }

    let proto_include_path = Path::new("..").canonicalize().unwrap();
    let proto_include_path_string = proto_include_path.to_str().unwrap();

    tonic_build::configure()
        .extern_path(".si.data", "::si_data::data")
        .extern_path(".si.account", "::si_account::protobuf")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute("in", "#[serde(rename = \"in\")]")
        .compile(PROTOS, &[proto_include_path_string])
        .unwrap();
}
