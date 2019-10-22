fn main() {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute("in", "#[serde(rename = \"in\")]")
        .compile(&["proto/ssh_key.proto"], &["proto"])
        .unwrap();
}
