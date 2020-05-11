use serde::{Deserialize, Serialize};

pub mod list_data;
pub mod relationship_data;
pub mod test_data;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiStorable {
    tenant_ids: Vec<String>,
    natural_key: Option<String>,
    type_name: Option<String>,
}
