use serde::{
    Deserialize,
    Serialize,
};
use si_id::FuncArgumentId;
use si_pkg::{
    LeafInputLocation as PkgLeafInputLocation,
    LeafKind as PkgLeafKind,
};
use strum::{
    AsRefStr,
    EnumIter,
};

use crate::{
    FuncBackendResponseType,
    attribute::path::AttributePath,
    func::argument::FuncArgumentKind,
    prop::PropPath,
    schema::variant::root_prop::RootPropChild,
};

/// This enum provides options for creating leaves underneath compatible subtrees of "/root" within
/// a [`SchemaVariant`](crate::SchemaVariant). Each compatible subtree starts with a
/// [`map`](crate::PropKind::Map) [`Prop`](crate::Prop) that can contain zero to many
/// [`object`](crate::PropKind::Object) entries. Each entry must leverage the same kind of
/// [`Func`](crate::Func) within the same [`map`](crate::PropKind::Map). The kind of
/// [`Func`](crate::Func) allowed corresponds to the [`LeafKind`].
#[remain::sorted]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, Serialize, Deserialize, Hash, AsRefStr)]
pub enum LeafKind {
    /// This variant corresponds to the "/root/code" subtree whose leaves leverage code generation
    /// [`Funcs`](crate::Func).
    CodeGeneration,
    /// This variant corresponds to the "/root/qualification" subtree whose leaves leverage
    /// qualification [`Funcs`](crate::Func).
    Qualification,
}

impl From<PkgLeafKind> for LeafKind {
    fn from(value: PkgLeafKind) -> Self {
        match value {
            PkgLeafKind::CodeGeneration => LeafKind::CodeGeneration,
            PkgLeafKind::Qualification => LeafKind::Qualification,
        }
    }
}

impl From<LeafKind> for PkgLeafKind {
    fn from(value: LeafKind) -> Self {
        match value {
            LeafKind::CodeGeneration => PkgLeafKind::CodeGeneration,
            LeafKind::Qualification => PkgLeafKind::Qualification,
        }
    }
}

/// This enum provides available child [`Prop`](crate::Prop) trees of [`RootProp`](crate::RootProp)
/// that can be used as "inputs" for [`Funcs`](crate::Func) on leaves.
///
/// _Note: not all [`children`](crate::RootPropChild) of [`RootProp`](crate::RootProp) can be used
/// as "inputs" in order to prevent cycles. This enum provides an approved subset of those
/// children_.
#[remain::sorted]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub enum LeafInputLocation {
    /// The input location corresponding to "/root/code".
    Code,
    /// The input location corresponding to "/root/deleted_at"
    DeletedAt,
    /// The input location corresponding to "/root/domain".
    Domain,
    /// The input location corresponding to "/root/resource".
    Resource,
    /// The input location corresponding to "/root/secrets".
    Secrets,
}

impl From<LeafInputLocation> for si_frontend_types::LeafInputLocation {
    fn from(value: LeafInputLocation) -> Self {
        match value {
            LeafInputLocation::Code => si_frontend_types::LeafInputLocation::Code,
            LeafInputLocation::DeletedAt => si_frontend_types::LeafInputLocation::DeletedAt,
            LeafInputLocation::Domain => si_frontend_types::LeafInputLocation::Domain,
            LeafInputLocation::Resource => si_frontend_types::LeafInputLocation::Resource,
            LeafInputLocation::Secrets => si_frontend_types::LeafInputLocation::Secrets,
        }
    }
}

impl From<LeafInputLocation> for AttributePath {
    fn from(value: LeafInputLocation) -> Self {
        AttributePath::from_json_pointer(match value {
            LeafInputLocation::Code => "/code",
            LeafInputLocation::DeletedAt => "/deleted_at",
            LeafInputLocation::Domain => "/domain",
            LeafInputLocation::Resource => "/resource",
            LeafInputLocation::Secrets => "/secrets",
        })
    }
}

impl From<&AttributePath> for Option<LeafInputLocation> {
    fn from(value: &AttributePath) -> Self {
        match value {
            AttributePath::JsonPointer(path) => match path.as_str() {
                "/code" => Some(LeafInputLocation::Code),
                "/deleted_at" => Some(LeafInputLocation::DeletedAt),
                "/domain" => Some(LeafInputLocation::Domain),
                "/resource" => Some(LeafInputLocation::Resource),
                "/secrets" => Some(LeafInputLocation::Secrets),
                _ => None,
            },
        }
    }
}

// We only want to allow converting an input location into a root prop child and root the other
// way around.
#[allow(clippy::from_over_into)]
impl Into<RootPropChild> for LeafInputLocation {
    fn into(self) -> RootPropChild {
        match self {
            LeafInputLocation::Code => RootPropChild::Code,
            LeafInputLocation::Domain => RootPropChild::Domain,
            LeafInputLocation::Resource => RootPropChild::Resource,
            LeafInputLocation::DeletedAt => RootPropChild::DeletedAt,
            LeafInputLocation::Secrets => RootPropChild::Secrets,
        }
    }
}

impl From<&PkgLeafInputLocation> for LeafInputLocation {
    fn from(value: &PkgLeafInputLocation) -> LeafInputLocation {
        match value {
            PkgLeafInputLocation::Code => Self::Code,
            PkgLeafInputLocation::Domain => Self::Domain,
            PkgLeafInputLocation::Resource => Self::Resource,
            PkgLeafInputLocation::DeletedAt => Self::DeletedAt,
            PkgLeafInputLocation::Secrets => Self::Secrets,
        }
    }
}

impl From<LeafInputLocation> for PkgLeafInputLocation {
    fn from(value: LeafInputLocation) -> Self {
        match value {
            LeafInputLocation::Code => Self::Code,
            LeafInputLocation::Domain => Self::Domain,
            LeafInputLocation::Resource => Self::Resource,
            LeafInputLocation::DeletedAt => Self::DeletedAt,
            LeafInputLocation::Secrets => Self::Secrets,
        }
    }
}
impl From<si_frontend_types::LeafInputLocation> for LeafInputLocation {
    fn from(value: si_frontend_types::LeafInputLocation) -> Self {
        match value {
            si_frontend_types::LeafInputLocation::Code => LeafInputLocation::Code,
            si_frontend_types::LeafInputLocation::DeletedAt => LeafInputLocation::DeletedAt,
            si_frontend_types::LeafInputLocation::Domain => LeafInputLocation::Domain,
            si_frontend_types::LeafInputLocation::Resource => LeafInputLocation::Resource,
            si_frontend_types::LeafInputLocation::Secrets => LeafInputLocation::Secrets,
        }
    }
}

impl From<PkgLeafInputLocation> for LeafInputLocation {
    fn from(value: PkgLeafInputLocation) -> LeafInputLocation {
        (&value).into()
    }
}

impl LeafInputLocation {
    pub fn arg_name(&self) -> &'static str {
        match self {
            LeafInputLocation::Code => "code",
            LeafInputLocation::Domain => "domain",
            LeafInputLocation::Resource => "resource",
            LeafInputLocation::DeletedAt => "deleted_at",
            LeafInputLocation::Secrets => "secrets",
        }
    }

    pub fn prop_path(&self) -> PropPath {
        PropPath::new(["root", self.arg_name()])
    }

    pub fn maybe_from_arg_name(arg_name: impl AsRef<str>) -> Option<Self> {
        Some(match arg_name.as_ref() {
            "domain" => LeafInputLocation::Domain,
            "code" => LeafInputLocation::Code,
            "resource" => LeafInputLocation::Resource,
            "deleted_at" => LeafInputLocation::DeletedAt,
            "secrets" => LeafInputLocation::Secrets,
            _ => return None,
        })
    }

    pub fn arg_kind(&self) -> FuncArgumentKind {
        match self {
            LeafInputLocation::Code
            | LeafInputLocation::Domain
            | LeafInputLocation::Resource
            | LeafInputLocation::Secrets => FuncArgumentKind::Object,
            LeafInputLocation::DeletedAt => FuncArgumentKind::String,
        }
    }
}

/// This struct provides the metadata necessary to provide "inputs" to [`Funcs`](crate::Func)
/// on leaves.
#[derive(Clone, Copy, Debug)]
pub struct LeafInput {
    /// The source location of the input.
    pub location: LeafInputLocation,
    /// The corresponding [`FuncArgumentId`](crate::FuncArgument) for the [`Func`](crate::Func).
    pub func_argument_id: FuncArgumentId,
}

impl LeafKind {
    /// Provides the names of the [`Map`](crate::PropKind::Map) and the child entry
    /// [`Props`](crate::Prop), respectively, for [`self`](Self).
    pub fn prop_names(&self) -> (&'static str, &'static str) {
        match self {
            LeafKind::CodeGeneration => ("code", "codeItem"),
            LeafKind::Qualification => ("qualification", "qualificationItem"),
        }
    }

    pub fn map_path(&self) -> AttributePath {
        match self {
            LeafKind::CodeGeneration => AttributePath::from_json_pointer("/code"),
            LeafKind::Qualification => AttributePath::from_json_pointer("/qualification"),
        }
    }
}

impl From<LeafKind> for FuncBackendResponseType {
    fn from(leaf_kind: LeafKind) -> Self {
        match leaf_kind {
            LeafKind::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            LeafKind::Qualification => FuncBackendResponseType::Qualification,
        }
    }
}
