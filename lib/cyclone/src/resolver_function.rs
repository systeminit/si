use crate::ComponentView;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ResolverFunctionComponent,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionComponent {
    pub data: ComponentView,
    pub parents: Vec<ComponentView>,
    // TODO: add widget data here (for example select's options)
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionResultSuccess {
    pub execution_id: String,
    pub data: Value,
    pub unset: bool,
    #[serde(default = "crate::timestamp")]
    pub timestamp: u64,
}
