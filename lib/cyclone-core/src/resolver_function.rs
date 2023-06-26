use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ComponentView;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ResolverFunctionComponent,
    pub response_type: ResolverFunctionResponseType,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionComponent {
    pub data: ComponentView,
    pub parents: Vec<ComponentView>,
    // TODO: add widget data here (for example select's options)
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
// Should be kept in sync with dal::func::backend::FuncBackendResponseType
pub enum ResolverFunctionResponseType {
    Action,
    Array,
    Boolean,
    CodeGeneration,
    Confirmation,
    Identity,
    Integer,
    Json,
    Map,
    Object,
    Qualification,
    Reconciliation,
    SchemaVariantDefinition,
    String,
    #[default]
    Unset,
    Validation,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionResultSuccess {
    pub execution_id: String,
    pub data: Value,
    pub unset: bool,
    pub timestamp: u64,
}
