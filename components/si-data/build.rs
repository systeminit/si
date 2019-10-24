use std::path::Path;

fn main() {
    println!("{}", file!());
    let proto_include_path = Path::new("..").canonicalize().unwrap();
    let proto_include_path_string = proto_include_path.to_str().unwrap();
    println!("{}", proto_include_path_string);
    let mut prost_build = prost_build::Config::new();
    prost_build
        .compile_protos(&["si-data/proto/data.proto"], &[proto_include_path_string])
        .expect("cannot compile protobufs");
}
