//! This module is strictly for types used by the si-fs fuse API client. The
//! types are deliberately not re-exported in the root, so that they don't get
//! mixed up with non si-fs types.

use serde::{Deserialize, Serialize};
use si_events::{ChangeSetId, FuncId, FuncKind, SchemaId, SchemaVariantId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeSet {
    pub name: String,
    pub id: ChangeSetId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateChangeSetRequest {
    pub name: String,
}

pub type CreateChangeSetResponse = ChangeSet;
pub type ListChangeSetsResponse = Vec<ChangeSet>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Schema {
    pub installed: bool,
    pub category: String,
    pub name: String,
    pub id: SchemaId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListVariantsResponse {
    pub locked: Option<SchemaVariantId>,
    pub unlocked: Option<SchemaVariantId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Func {
    pub id: FuncId,
    pub kind: FuncKind,
    pub name: String,
    pub is_locked: bool,
    pub code_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetFuncs {
    pub locked: Option<Func>,
    pub unlocked: Option<Func>,
}

pub fn kind_to_string(kind: FuncKind) -> String {
    match kind {
        FuncKind::Action => "action",
        FuncKind::Attribute => "attribute",
        FuncKind::Authentication => "authentication",
        FuncKind::CodeGeneration => "code-generation",
        FuncKind::Intrinsic => "intrinsic",
        FuncKind::Qualification => "qualification",
        FuncKind::SchemaVariantDefinition => "asset-definition",
        FuncKind::Unknown => "unknown",
        FuncKind::Management => "management",
    }
    .into()
}

pub fn kind_pluralized_to_string(kind: FuncKind) -> String {
    match kind {
        FuncKind::Action => "actions",
        FuncKind::Attribute => "attributes",
        FuncKind::Authentication => "authentications",
        FuncKind::CodeGeneration => "code-generations",
        FuncKind::Intrinsic => "intrinsics",
        FuncKind::Qualification => "qualifications",
        FuncKind::SchemaVariantDefinition => "asset-definitions",
        FuncKind::Unknown => "unknowns",
        FuncKind::Management => "managements",
    }
    .into()
}

pub fn kind_from_string(s: &str) -> Option<FuncKind> {
    Some(match s {
        "action" => FuncKind::Action,
        "attribute" => FuncKind::Attribute,
        "authentication" => FuncKind::Authentication,
        "code-generation" => FuncKind::CodeGeneration,
        "intrinsic" => FuncKind::Intrinsic,
        "qualification" => FuncKind::Qualification,
        "asset-definition" => FuncKind::SchemaVariantDefinition,
        "unknown" => FuncKind::Unknown,
        "management" => FuncKind::Management,
        _ => return None,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantQuery {
    pub unlocked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetFuncCodeRequest {
    pub code: String,
}
