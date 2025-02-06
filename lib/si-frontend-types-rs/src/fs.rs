//! This module is strictly for types used by the si-fs fuse API client. The
//! types are deliberately not re-exported in the root, so that they don't get
//! mixed up with non si-fs types.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use si_events::{ChangeSetId, FuncId, FuncKind, SchemaId, SchemaVariantId};

pub use si_events::ActionKind;

use crate::{ComponentType, FuncArgumentKind, LeafInputLocation};

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
    pub bindings_size: u64,
    pub types_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetFuncs {
    pub locked: Option<Func>,
    pub unlocked: Option<Func>,
    pub locked_attrs_size: u64,
    pub locked_bindings_size: u64,
    pub unlocked_attrs_size: u64,
    pub unlocked_bindings_size: u64,
    pub types_size: u64,
}

impl AssetFuncs {
    pub fn extend_code_sizes(&mut self, by: u64) {
        if let Some(func) = self.locked.as_mut() {
            func.code_size += by;
        }
        if let Some(func) = self.unlocked.as_mut() {
            func.code_size += by;
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateFuncRequest {
    pub name: String,
    pub binding: Binding,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaAttributes {
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub color: String,
    pub component_type: ComponentType,
}

impl SchemaAttributes {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn byte_size(&self) -> u64 {
        self.to_vec_pretty().ok().map(|vec| vec.len()).unwrap_or(0) as u64
    }

    pub fn to_vec_pretty(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaAttributesResponse {
    pub locked: Option<SchemaAttributes>,
    pub unlocked: Option<SchemaAttributes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AttributeOutputTo {
    OutputSocket(String),
    Prop(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributeFuncInput {
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub input: Option<AttributeInputFrom>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AttributeInputFrom {
    InputSocket(String),
    Prop(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Binding {
    Action {
        kind: ActionKind,
    },
    Attribute {
        output_to: AttributeOutputTo,
        // BTreeMap to preserve order
        inputs: BTreeMap<String, AttributeFuncInput>,
    },
    Authentication,
    CodeGeneration {
        inputs: Vec<LeafInputLocation>,
    },
    Management {
        managed_schemas: Option<Vec<String>>,
    },
    Qualification {
        inputs: Vec<LeafInputLocation>,
    },
}

impl Binding {
    pub fn kind_matches(&self, func_kind: FuncKind) -> bool {
        match self {
            Binding::Action { .. } => func_kind == FuncKind::Action,
            Binding::Attribute { .. } => func_kind == FuncKind::Attribute,
            Binding::Authentication => func_kind == FuncKind::Authentication,
            Binding::CodeGeneration { .. } => func_kind == FuncKind::CodeGeneration,
            Binding::Management { .. } => func_kind == FuncKind::Management,
            Binding::Qualification { .. } => func_kind == FuncKind::Qualification,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bindings {
    pub bindings: Vec<Binding>,
}

impl Bindings {
    pub fn kind_matches(&self, func_kind: FuncKind) -> bool {
        !self
            .bindings
            .iter()
            .any(|binding| !binding.kind_matches(func_kind))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        Ok(Self {
            bindings: serde_json::from_slice(bytes)?,
        })
    }

    pub fn byte_size(&self) -> u64 {
        self.to_vec_pretty().ok().map(|vec| vec.len()).unwrap_or(0) as u64
    }

    pub fn to_vec_pretty(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(&self.bindings)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSchemaRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSchemaResponse {
    pub schema_id: SchemaId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FsApiError {
    ChangeSetInactive(ChangeSetId),
    ResourceNotFound,
    InternalServerError(String),
}

impl fmt::Display for FsApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsApiError::ChangeSetInactive(change_set_id) => {
                write!(f, "ChangeSetInactive({change_set_id})")
            }
            FsApiError::ResourceNotFound => write!(f, "ResourceNotFound"),
            FsApiError::InternalServerError(error) => write!(f, "InternalServerError: {error}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetFuncBindingsRequest {
    pub bindings: Bindings,
    pub is_attaching_existing: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PropIdentityBinding {
    InputSocket(String),
    Prop(String),
    Unset,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SocketIdentityBinding {
    Prop(String),
    Unset,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentityBindings {
    pub props: BTreeMap<String, PropIdentityBinding>,
    pub output_sockets: BTreeMap<String, SocketIdentityBinding>,
}

impl IdentityBindings {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn byte_size(&self) -> u64 {
        self.to_vec_pretty().ok().map(|vec| vec.len()).unwrap_or(0) as u64
    }

    pub fn to_vec_pretty(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(&self)
    }
}
