//! This module contains all "leaves" that can be created underneath [`RootProp`](crate::RootProp)
//! subtrees for a [`SchemaVariant`](crate::SchemaVariant). In this domain, a "leaf" is considered
//! to an entry of a immediate child [`map`](crate::PropKind::Map) underneath "/root".

use strum_macros::EnumIter;

use crate::func::argument::{FuncArgumentId, FuncArgumentKind};
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    AttributeContext, AttributePrototype, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueError, ComponentId, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncError, FuncId, PropId, RootPropChild, SchemaVariant,
    SchemaVariantId, StandardModel,
};
use si_pkg::{LeafInputLocation as PkgLeafInputLocation, LeafKind as PkgLeafKind};

/// This enum provides options for creating leaves underneath compatible subtrees of "/root" within
/// a [`SchemaVariant`](crate::SchemaVariant). Each compatible subtree starts with a
/// [`map`](crate::PropKind::Map) [`Prop`](crate::Prop) that can contain zero to many
/// [`object`](crate::PropKind::Object) entries. Each entry must leverage the same kind of
/// [`Func`](crate::Func) within the same [`map`](crate::PropKind::Map). The kind of
/// [`Func`](crate::Func) allowed corresponds to the [`LeafKind`].
#[derive(Clone, Copy, Debug, EnumIter)]
pub enum LeafKind {
    /// This variant corresponds to the "/root/code" subtree whose leaves leverage code generation
    /// [`Funcs`](crate::Func).
    CodeGeneration,
    /// This variant corresponds to the "/root/qualification" subtree whose leaves leverage
    /// qualification [`Funcs`](crate::Func).
    Qualification,
    /// This variant corresponds to the "/root/confirmation" subtree whose leaves leverage
    /// confirmation [`Funcs`](crate::Func).
    Confirmation,
}

impl From<PkgLeafKind> for LeafKind {
    fn from(value: PkgLeafKind) -> Self {
        match value {
            PkgLeafKind::CodeGeneration => LeafKind::CodeGeneration,
            PkgLeafKind::Confirmation => LeafKind::Confirmation,
            PkgLeafKind::Qualification => LeafKind::Qualification,
        }
    }
}

impl From<LeafKind> for PkgLeafKind {
    fn from(value: LeafKind) -> Self {
        match value {
            LeafKind::CodeGeneration => PkgLeafKind::CodeGeneration,
            LeafKind::Confirmation => PkgLeafKind::Confirmation,
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
#[derive(Clone, Copy, Debug)]
pub enum LeafInputLocation {
    /// The input location corresponding to "/root/code".
    Code,
    /// The input location corresponding to "/root/domain".
    Domain,
    /// The input location corresponding to "/root/resource".
    Resource,
    /// The input location corresponding to "/root/deleted_at"
    DeletedAt,
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
        }
    }
}

impl From<&PkgLeafInputLocation> for LeafInputLocation {
    fn from(value: &PkgLeafInputLocation) -> LeafInputLocation {
        match value {
            PkgLeafInputLocation::Code => LeafInputLocation::Code,
            PkgLeafInputLocation::Domain => LeafInputLocation::Domain,
            PkgLeafInputLocation::Resource => LeafInputLocation::Resource,
            PkgLeafInputLocation::DeletedAt => LeafInputLocation::DeletedAt,
        }
    }
}

impl LeafInputLocation {
    pub fn arg_name(&self) -> &'static str {
        match self {
            LeafInputLocation::Code => "code",
            LeafInputLocation::Domain => "domain",
            LeafInputLocation::Resource => "resource",
            LeafInputLocation::DeletedAt => "deleted_at",
        }
    }

    pub fn arg_kind(&self) -> FuncArgumentKind {
        match self {
            LeafInputLocation::Code | LeafInputLocation::Domain | LeafInputLocation::Resource => {
                FuncArgumentKind::Object
            }
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
            LeafKind::Confirmation => ("confirmation", "confirmationItem"),
        }
    }
}

impl From<LeafKind> for FuncBackendResponseType {
    fn from(leaf_kind: LeafKind) -> Self {
        match leaf_kind {
            LeafKind::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            LeafKind::Qualification => FuncBackendResponseType::Qualification,
            LeafKind::Confirmation => FuncBackendResponseType::Confirmation,
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
    ) -> SchemaVariantResult<(PropId, AttributePrototype)> {
        if schema_variant_id.is_none() {
            return Err(SchemaVariantError::InvalidSchemaVariant);
        }

        // Ensure the func matches what we need.
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        if func.backend_kind() != &FuncBackendKind::JsAttribute {
            return Err(SchemaVariantError::LeafFunctionMustBeJsAttribute(
                *func.id(),
            ));
        }
        if func.backend_response_type() != &leaf_kind.into() {
            return Err(SchemaVariantError::LeafFunctionMismatch(
                *func.backend_response_type(),
                leaf_kind,
            ));
        }

        // We only need to finalize once since we are adding a leaf to a known descendant of the
        // root prop.
        let mut schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;
        if !schema_variant.finalized_once() {
            schema_variant.finalize(ctx, None).await?;
        }

        // Assemble the values we need to insert an object into the map.
        let item_prop =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

        // NOTE(nick): we should consider getting the parent and the item at the same time.
        let map_prop = item_prop
            .parent_prop(ctx)
            .await?
            .ok_or_else(|| SchemaVariantError::ParentPropNotFound(*item_prop.id()))?;
        let map_attribute_read_context =
            AttributeReadContext::default_with_prop_and_component_id(*map_prop.id(), component_id);
        let map_attribute_value = AttributeValue::find_for_context(ctx, map_attribute_read_context)
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(
                map_attribute_read_context,
            ))?;
        let insert_attribute_context = AttributeContext::builder()
            .set_prop_id(*item_prop.id())
            .set_component_id(component_id.unwrap_or(ComponentId::NONE))
            .to_context()?;

        // Insert an item into the map and setup its function. The new entry is named after the func
        // name since func names must be unique for a given tenancy and visibility. If that changes,
        // then this will break.
        let inserted_attribute_value_id = AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *map_attribute_value.id(),
            Some(serde_json::json![{}]),
            Some(func.name().to_string()),
        )
        .await?;
        let inserted_attribute_value = AttributeValue::get_by_id(ctx, &inserted_attribute_value_id)
            .await?
            .ok_or_else(|| {
                AttributeValueError::NotFound(inserted_attribute_value_id, *ctx.visibility())
            })?;
        let mut inserted_attribute_prototype = inserted_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        inserted_attribute_prototype
            .set_func_id(ctx, func_id)
            .await?;

        for input in inputs {
            let input_internal_provider =
                SchemaVariant::find_root_child_implicit_internal_provider(
                    ctx,
                    schema_variant_id,
                    input.location.into(),
                )
                .await?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *inserted_attribute_prototype.id(),
                input.func_argument_id,
                *input_internal_provider.id(),
            )
            .await?;
        }

        // Return the prop id for the entire map so that its implicit internal provider can be
        // used for intelligence functions.
        Ok((*map_prop.id(), inserted_attribute_prototype))
    }
}
