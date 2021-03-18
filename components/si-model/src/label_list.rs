use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LabelListItem {
    pub label: String,
    pub value: String,
}

pub type LabelList = Vec<LabelListItem>;
