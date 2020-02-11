fn main() {
    tonic_build::configure()
        .extern_path(".si.data", "::si_data::data")
        .extern_path(".si.account", "::si_account::protobuf")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile(
            &["si-external-api-gateway/proto/si.external_api_gateway.aws.ec2.proto"],
            &["../.."],
        )
        .unwrap();
}
