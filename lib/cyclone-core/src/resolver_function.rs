use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::{
    ComponentView,
    before::BeforeFunction,
    request::CycloneRequestable,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ResolverFunctionComponent,
    pub response_type: ResolverFunctionResponseType,
    pub code_base64: String,
    pub before: Vec<BeforeFunction>,
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
    Debug,
    Float,
    Identity,
    Integer,
    Json,
    Management,
    Map,
    Object,
    Qualification,
    String,
    #[default]
    Unset,
    Void,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionResultSuccess {
    pub execution_id: String,
    pub data: Value,
    pub unset: bool,
    pub timestamp: u64,
}

impl CycloneRequestable for ResolverFunctionRequest {
    type Response = ResolverFunctionResultSuccess;

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        "resolverfunction"
    }

    fn websocket_path(&self) -> &str {
        "/execute/resolver"
    }

    fn inc_run_metric(&self) {
        metric!(counter.function_run.resolver = 1);
    }

    fn dec_run_metric(&self) {
        metric!(counter.function_run.resolver = -1);
    }
}
