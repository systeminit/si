use std::path::Path;

const PROTOS: &[&str] = &["si-registry/proto/si.kubernetes.proto"];

fn main() {
    println!("cargo:rerun-if-changes=../si-cea");
    println!("cargo:rerun-if-changed=Cargo.toml");
    for proto in PROTOS {
        println!("cargo:rerun-if-changed=../{}", proto);
    }

    let proto_include_path = Path::new("..").canonicalize().unwrap();
    let proto_include_path_string = proto_include_path.to_str().unwrap();

    tonic_build::configure()
        .extern_path(".si.data", "::si_data::protobuf")
        .extern_path(".si.account", "::si_account::protobuf")
        .extern_path(".si.cea", "::si_cea::protobuf")
        .extern_path(".si.data", "::si_data::data")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .type_attribute("Entity", "#[serde(default)]")
        .type_attribute("EntityEvent", "#[serde(default)]")
        .field_attribute("in", "#[serde(rename = \"in\")]")
        .compile(PROTOS, &[proto_include_path_string])
        .unwrap();
}
