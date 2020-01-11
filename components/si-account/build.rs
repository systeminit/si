// , deserialize_with = \"crate::serde_enum::key_type_enum_d\")]
fn main() {
    tonic_build::configure()
        .extern_path(".si.data", "::si_data::data")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute(
            "tenant.kind",
            "#[serde(serialize_with = \"crate::serde_enum::tenantkind_enum_s\", deserialize_with = \"crate::serde_enum::tenantkind_enum_d\")]",
        )
        .field_attribute("in", "#[serde(rename = \"in\")]")
        .compile(&["si-account/proto/si.account.proto"], &[".."])
        .unwrap();
}
