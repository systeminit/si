use std::path::Path;

fn main() {
    println!("{}", file!());
    let proto_include_path = Path::new("..").canonicalize().unwrap();
    let proto_include_path_string = proto_include_path.to_str().unwrap();
    println!("{}", proto_include_path_string);
    let mut prost_build = prost_build::Config::new();
    prost_build
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile_protos(&["si-data/proto/si.data.proto"], &[proto_include_path_string])
        .expect("cannot compile protobufs");
}
