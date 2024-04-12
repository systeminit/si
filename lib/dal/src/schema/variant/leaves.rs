//! This module contains all "leaves" that can be created underneath [`RootProp`](crate::RootProp)
//! subtrees for a [`SchemaVariant`](crate::SchemaVariant). In this domain, a "leaf" is considered
//! to an entry of a immediate child [`map`](crate::PropKind::Map) underneath "/root".

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use telemetry::prelude::*;

use crate::attribute::prototype::argument::AttributePrototypeArgument;
use crate::workspace_snapshot::edge_weight::EdgeWeightKind;
use crate::{
    AttributePrototype, AttributePrototypeId, ComponentId, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncId, Prop, PropId, SchemaVariant, SchemaVariantId,
};
use si_pkg::{LeafInputLocation as PkgLeafInputLocation, LeafKind as PkgLeafKind};

use crate::func::argument::{FuncArgumentId, FuncArgumentKind};
use crate::prop::PropPath;
use crate::schema::variant::root_prop::RootPropChild;

use super::{SchemaVariantError, SchemaVariantResult};

/// This enum provides options for creating leaves underneath compatible subtrees of "/root" within
/// a [`SchemaVariant`](crate::SchemaVariant). Each compatible subtree starts with a
/// [`map`](crate::PropKind::Map) [`Prop`](crate::Prop) that can contain zero to many
/// [`object`](crate::PropKind::Object) entries. Each entry must leverage the same kind of
/// [`Func`](crate::Func) within the same [`map`](crate::PropKind::Map). The kind of
/// [`Func`](crate::Func) allowed corresponds to the [`LeafKind`].
#[remain::sorted]
#[derive(Clone, Copy, Debug, EnumIter)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
}

impl From<LeafKind> for FuncBackendResponseType {
    fn from(leaf_kind: LeafKind) -> Self {
        match leaf_kind {
            LeafKind::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            LeafKind::Qualification => FuncBackendResponseType::Qualification,
        }
    }
}

impl SchemaVariant {
    /// Insert an [`object`](crate::PropKind::Object) entry into a "/root" subtree of
    /// [`map`](crate::PropKind::Map) with a [`Func`](crate::Func) that matches the provided
    /// [`LeafKind`] in order to populate the subtree entry.
    ///
    /// The [`PropId`](crate::Prop) for the child [`map`](crate::PropKind::Map) of "/root"
    /// corresponding to the [`LeafKind`] is returned.
    pub async fn add_leaf(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
        component_id: Option<ComponentId>,
        leaf_kind: LeafKind,
        inputs: Vec<LeafInput>,
    ) -> SchemaVariantResult<(PropId, AttributePrototypeId)> {
        // Ensure the func matches what we need.
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        if func.backend_kind != FuncBackendKind::JsAttribute {
            return Err(SchemaVariantError::LeafFunctionMustBeJsAttribute(func.id));
        }
        if func.backend_response_type != leaf_kind.into() {
            return Err(SchemaVariantError::LeafFunctionMismatch(
                func_id,
                func.backend_response_type,
                leaf_kind,
            ));
        }

        if component_id.is_some() {
            unimplemented!("component context for leaves not yet implemented in graph version");
        }

        let item_prop_id =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

        let map_prop_id = Prop::parent_prop_id_by_id(ctx, item_prop_id)
            .await?
            .ok_or_else(|| SchemaVariantError::LeafMapPropNotFound(item_prop_id))?;

        if let Some(prototype_id) =
            AttributePrototype::find_for_prop(ctx, item_prop_id, &None).await?
        {
            debug!(%prototype_id, %item_prop_id, "removing None proto");
            AttributePrototype::remove(ctx, prototype_id).await?;
        }

        let key = Some(func.name.to_owned());
        if let Some(prototype_id) =
            AttributePrototype::find_for_prop(ctx, item_prop_id, &key).await?
        {
            debug!(%prototype_id, %item_prop_id, "removing {:?} proto", &key);
            AttributePrototype::remove(ctx, prototype_id).await?;
        }

        let attribute_prototype_id = AttributePrototype::new(ctx, func_id).await?.id();

        Prop::add_edge_to_attribute_prototype(
            ctx,
            item_prop_id,
            attribute_prototype_id,
            EdgeWeightKind::Prototype(key),
        )
        .await?;

        for input in inputs {
            let input_prop_id = SchemaVariant::find_root_child_prop_id(
                ctx,
                schema_variant_id,
                input.location.into(),
            )
            .await?;

            let apa = AttributePrototypeArgument::new(
                ctx,
                attribute_prototype_id,
                input.func_argument_id,
            )
            .await?;

            apa.set_value_from_prop_id(ctx, input_prop_id).await?;
        }

        Ok((map_prop_id, attribute_prototype_id))
    }
}
