use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::request::CycloneRequestable;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionResultSuccess {
    pub execution_id: String,
    pub definition: serde_json::Value,
    // Collects the error if the function throws
    #[serde(default)]
    pub error: Option<String>,
}

impl CycloneRequestable for SchemaVariantDefinitionRequest {
    type Response = SchemaVariantDefinitionResultSuccess;

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        "schemaVariantDefinition"
    }

    fn websocket_path(&self) -> &str {
        "/execute/schema_variant_definition"
    }

    fn inc_run_metric(&self) {
        metric!(counter.function_run.schema_variant_definition = 1);
    }

    fn dec_run_metric(&self) {
        metric!(counter.function_run.schema_variant_definition = -1);
    }
}
