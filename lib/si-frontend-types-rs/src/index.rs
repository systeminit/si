use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontEndObjectRequest {
    pub kind: String,
    pub id: String,
    pub checksum: Option<String>,
}
