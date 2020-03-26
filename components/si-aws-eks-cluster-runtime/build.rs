const PROTOS: &[&str] = &["si-aws-eks-cluster-runtime/proto/si.aws_eks_cluster_runtime.proto"];

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    for proto in PROTOS {
        println!("cargo:rerun-if-changed=../{}", proto);
    }

    tonic_build::configure()
        .extern_path(".si.data", "::si_data::data")
        .extern_path(".si.account", "::si_account::protobuf")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .type_attribute("Entity", "#[serde(default)]")
        .type_attribute("EntityEvent", "#[serde(default)]")
        .field_attribute("in", "#[serde(rename = \"in\")]")
        .compile(PROTOS, &[".."])
        .unwrap();
}
