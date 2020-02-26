const PROTOS: &[&str] = &[
    "si-external-api-gateway/proto/si.external_api_gateway.aws.ec2.proto",
    "si-external-api-gateway/proto/si.external_api_gateway.aws.eks.proto",
];

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    for proto in PROTOS {
        println!("cargo:rerun-if-changed=../../{}", proto);
    }

    tonic_build::configure()
        .extern_path(".si.data", "::si_data::data")
        .extern_path(".si.account", "::si_account::protobuf")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .type_attribute(".", "#[derive(typed_builder::TypedBuilder)]")
        .compile(PROTOS, &["../.."])
        .unwrap();
}
