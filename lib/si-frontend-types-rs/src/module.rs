use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use si_events::SchemaVariantId;

pub use module_index_types::BuiltinsDetailsResponse as BuiltinModules;
pub use module_index_types::LatestModuleResponse as LatestModule;
pub use module_index_types::ModuleDetailsResponse as ModuleDetails;

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncedModules {
    pub upgradeable: HashMap<SchemaVariantId, LatestModule>,
    pub contributable: Vec<SchemaVariantId>,
}

impl SyncedModules {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModuleContributeRequest {
    pub name: String,
    pub version: String,
    pub schema_variant_id: SchemaVariantId,
    pub is_private_module: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSummary {
    pub name: String,
    pub hash: String,
}
