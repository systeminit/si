use serde::{
    Deserialize,
    Serialize,
};
use si_id::PropId;

/// Schema information for a property, used by Luminork API responses.
/// This is a shared type between luminork-server and cached schema variants.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::DefinitionChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct PropSchemaV1 {
    pub prop_id: PropId,
    pub name: String,
    pub prop_type: String,
    pub description: Option<String>,
    /// Recursive field: Uses field name + type string to break recursion cycles
    /// This prevents infinite loops during static initialization while maintaining
    /// schema change detection (e.g., changing Vec<PropSchemaV1> to HashMap<String, PropSchemaV1>)
    #[definition_checksum(recursive_definition)]
    pub children: Option<Vec<PropSchemaV1>>,
    // New fields from PropSpecData (excluding func/widget/ui fields)
    pub validation_format: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub hidden: Option<bool>,
    pub doc_link: Option<String>,
}
