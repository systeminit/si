//! This module contains [`SchemaVariant`](crate::SchemaVariant), which is t/he "class" of a
//! [`Component`](crate::Component).

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::prop::PropPath;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::definition::{SchemaVariantDefinitionError, SchemaVariantDefinitionId};
use crate::schema::variant::root_prop::component_type::ComponentType;
use crate::schema::variant::root_prop::SiPropChild;
use crate::standard_model::{object_from_row, option_object_from_row};
use crate::AttributePrototypeArgument;
use crate::{
    func::{
        argument::{FuncArgument, FuncArgumentError},
        binding::FuncBindingError,
        binding_return_value::FuncBindingReturnValueId,
    },
    impl_standard_model, pk,
    schema::{RootProp, SchemaError},
    socket::{Socket, SocketError, SocketId},
    standard_model::{self, objects_from_rows},
    standard_model_accessor, standard_model_belongs_to, standard_model_many_to_many,
    AttributeContextBuilderError, AttributePrototype, AttributePrototypeArgumentError,
    AttributePrototypeError, AttributeReadContext, AttributeValue, AttributeValueError,
    AttributeValueId, BuiltinsError, Component, ComponentError, ComponentId, DalContext,
    ExternalProvider, ExternalProviderError, Func, FuncBackendResponseType, FuncBindingReturnValue,
    FuncError, FuncId, HistoryEventError, InternalProvider, Prop, PropError, PropId, PropKind,
    ReconciliationPrototypeError, RootPropChild, Schema, SchemaId, SocketArity, StandardModel,
    StandardModelError, Tenancy, Timestamp, TransactionsError, ValidationPrototypeError,
    Visibility, WorkspacePk, WsEventError,
};

use self::leaves::{LeafInput, LeafInputLocation, LeafKind};

pub mod definition;
pub mod leaves;
pub mod root_prop;

const ALL_FUNCS: &str = include_str!("../queries/schema_variant/all_related_funcs.sql");
const ALL_PROPS: &str = include_str!("../queries/schema_variant/all_props.sql");
const FIND_ROOT_PROP: &str = include_str!("../queries/schema_variant/find_root_prop.sql");
const FIND_LEAF_ITEM_PROP: &str = include_str!("../queries/schema_variant/find_leaf_item_prop.sql");
const FIND_ROOT_CHILD_IMPLICIT_INTERNAL_PROVIDER: &str =
    include_str!("../queries/schema_variant/find_root_child_implicit_internal_provider.sql");
const LIST_ROOT_SI_CHILD_PROPS: &str =
    include_str!("../queries/schema_variant/list_root_si_child_props.sql");
const SECRET_DEFINING_SCHEMA_VARIANTS: &str =
    include_str!("../queries/schema_variant/secret_defining_schema_variants.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    /// Not parent [`AttributeValue`](crate::AttributeValue) was found for the specified
    /// [`AttributeValueId`](crate::AttributeValue).
    #[error("no parent found for attribute value: {0}")]
    AttributeValueDoesNotHaveParent(AttributeValueId),
    /// An [`AttributeValue`](crate::AttributeValue) could not be found for the specified
    /// [`AttributeReadContext`](crate::AttributeReadContext).
    #[error("attribute value not found for attribute read context: {0:?}")]
    AttributeValueNotFoundForContext(Box<AttributeReadContext>),
    #[error(transparent)]
    Builtins(#[from] Box<BuiltinsError>),
    #[error(transparent)]
    Component(#[from] Box<ComponentError>),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("can neither provide children nor entry for primitive with name: ({0})")]
    FoundChildrenAndEntryForPrimitive(String),
    #[error("cannot provide children for array with name: ({0})")]
    FoundChildrenForArray(String),
    #[error("cannot provide children for primitive with name: ({0})")]
    FoundChildrenForPrimitive(String),
    #[error("cannot provide entry for object with name: ({0})")]
    FoundEntryForObject(String),
    #[error("cannot provide entry for primitive with name: ({0})")]
    FoundEntryForPrimitive(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func binding return value not found {0}")]
    FuncBindingReturnValueNotFound(FuncBindingReturnValueId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("must provide valid schema variant, found unset schema variant id")]
    InvalidSchemaVariant,
    #[error("leaf function response type ({0}) must match leaf kind ({0})")]
    LeafFunctionMismatch(FuncBackendResponseType, LeafKind),
    #[error("leaf function ({0}) must be JsAttribute")]
    LeafFunctionMustBeJsAttribute(FuncId),
    #[error("link not found in doc links map for doc link ref: {0}")]
    LinkNotFoundForDocLinkRef(String),
    #[error("must provide children for object with name: ({0})")]
    MissingChildrenForObject(String),
    #[error("must provide entry for array with name: ({0})")]
    MissingEntryForArray(String),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("Schema is missing for SchemaVariant {0}")]
    MissingSchema(SchemaVariantId),
    #[error("cannot use doc link and doc link ref for prop definition name: ({0})")]
    MultipleDocLinksProvided(String),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("schema variant not found: {0}")]
    NotFound(SchemaVariantId),
    #[error("parent prop not found for prop id: {0}")]
    ParentPropNotFound(PropId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    /// This variant indicates that a [`Prop`](crate::Prop) or [`PropId`](crate::Prop) was not
    /// found. However, it does not _describe_ the attempt to locate the object in question. The
    /// "json pointer" piece is purely meant to help describe the location.
    #[error("prop not found corresponding to the following json pointer: {0}")]
    PropNotFound(&'static str),
    #[error("cannot find prop at path {1} for SchemaVariant {0} and Visibility {2:?}")]
    PropNotFoundAtPath(SchemaVariantId, String, Visibility),
    #[error("prop not found in cache for name ({0}) and parent prop id ({1})")]
    PropNotFoundInCache(String, PropId),
    #[error("reconciliation prototype: {0}")]
    ReconciliationPrototype(#[from] ReconciliationPrototypeError),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant definition error")]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("std error: {0}")]
    Std(#[from] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

pk!(SchemaVariantPk);
pk!(SchemaVariantId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    ui_hidden: bool,
    name: String,
    /// The [`RootProp`](crate::RootProp) for [`self`](Self).
    root_prop_id: Option<PropId>,
    schema_variant_definition_id: Option<SchemaVariantDefinitionId>,
    link: Option<String>,
    // NOTE(nick): we may want to replace this with a better solution. We use this to ensure
    // components are not created unless the variant has been finalized at least once.
    finalized_once: bool,
}

impl_standard_model! {
    model: SchemaVariant,
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    table_name: "schema_variants",
    history_event_label_base: "schema_variant",
    history_event_message_name: "Schema Variant"
}

impl SchemaVariant {
    /// Create a [`SchemaVariant`](Self) with a [`RootProp`](crate::schema::RootProp).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        schema_id: SchemaId,
        name: impl AsRef<str>,
    ) -> SchemaVariantResult<(Self, RootProp)> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM schema_variant_create_v1($1, $2, $3)",
                &[ctx.tenancy(), ctx.visibility(), &name],
            )
            .await?;
        let mut object: SchemaVariant = standard_model::finish_create_from_row(ctx, row).await?;
        let root_prop = object.create_and_set_root_prop(ctx, schema_id).await?;
        object.set_schema(ctx, &schema_id).await?;

        let (identity_func, identity_func_binding, identity_func_binding_return_value) =
            Func::identity_with_binding_and_return_value(ctx).await?;

        // all nodes can be turned into frames therefore, they will need a frame input socket
        // the UI itself will determine if this socket is available to be connected
        let (_frame_internal_provider, _input_socket) = InternalProvider::new_explicit_with_socket(
            ctx,
            *object.id(),
            "Frame",
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
            SocketArity::Many,
            true,
        )
        .await?;

        let (_output_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            schema_id,
            *object.id(),
            "Frame",
            None,
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
            SocketArity::One,
            true,
        )
        .await?;

        Ok((object, root_prop))
    }

    pub async fn is_builtin(&self, ctx: &DalContext) -> SchemaVariantResult<bool> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT id FROM schema_variants WHERE id = $1 and tenancy_workspace_pk = $2 LIMIT 1",
                &[self.id(), &WorkspacePk::NONE],
            )
            .await?;

        Ok(row.is_some())
    }

    /// This _idempotent_ function "finalizes" a [`SchemaVariant`].
    ///
    /// Once a [`SchemaVariant`] has had all of its [`Props`](crate::Prop) created, there are a few
    /// things that need to happen before it is usable:
    ///
    /// * Create the default [`AttributePrototypes`](crate::AttributePrototype) and
    ///   [`AttributeValues`](crate::AttributeValue).
    /// * Create the _internally consuming_ [`InternalProviders`](crate::InternalProvider)
    ///   corresponding to every [`Prop`](crate::Prop) in the [`SchemaVariant`] that is not a
    ///   descendant of an Array or a Map.
    ///
    /// This method **MUST** be called once all the [`Props`](Prop) have been created for the
    /// [`SchemaVariant`]. It can be called multiple times while [`Props`](Prop) are being created,
    /// but it must be called once after all [`Props`](Prop) have been created.
    pub async fn finalize(
        &mut self,
        ctx: &DalContext,
        component_type: Option<ComponentType>,
    ) -> SchemaVariantResult<()> {
        let total_start = std::time::Instant::now();

        Self::create_default_prototypes_and_values(ctx, self.id).await?;
        Self::create_implicit_internal_providers(ctx, self.id).await?;
        if !self.finalized_once() {
            self.set_finalized_once(ctx, true).await?;
        }

        // Default to the standard "component" component type.
        let component_type = match component_type {
            Some(component_type) => component_type,
            None => ComponentType::Component,
        };

        // Find props that we need to set defaults on for _all_ schema variants.
        // FIXME(nick): use the enum and create an appropriate query.
        let mut maybe_type_prop_id = None;
        let mut maybe_protected_prop_id = None;
        for root_si_child_prop in Self::list_root_si_child_props(ctx, self.id).await? {
            if root_si_child_prop.name() == "type" {
                maybe_type_prop_id = Some(*root_si_child_prop.id())
            } else if root_si_child_prop.name() == "protected" {
                maybe_protected_prop_id = Some(*root_si_child_prop.id())
            }
        }
        let type_prop_id =
            maybe_type_prop_id.ok_or(SchemaVariantError::PropNotFound("/root/si/type"))?;
        let protected_prop_id = maybe_protected_prop_id
            .ok_or(SchemaVariantError::PropNotFound("/root/si/protected"))?;

        // Set the default type of the schema variant.
        let attribute_read_context = AttributeReadContext::default_with_prop(type_prop_id);
        let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
            .await?
            .ok_or_else(|| {
                SchemaVariantError::AttributeValueNotFoundForContext(attribute_read_context.into())
            })?;
        let parent_attribute_value = attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| {
                SchemaVariantError::AttributeValueDoesNotHaveParent(*attribute_value.id())
            })?;
        let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
        AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(*parent_attribute_value.id()),
            context,
            Some(serde_json::to_value(component_type)?),
            None,
        )
        .await?;

        // Ensure _all_ schema variants are not protected by default.
        let attribute_read_context = AttributeReadContext::default_with_prop(protected_prop_id);
        let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
            .await?
            .ok_or_else(|| {
                SchemaVariantError::AttributeValueNotFoundForContext(attribute_read_context.into())
            })?;
        let parent_attribute_value = attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| {
                SchemaVariantError::AttributeValueDoesNotHaveParent(*attribute_value.id())
            })?;
        let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
        AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(*parent_attribute_value.id()),
            context,
            Some(serde_json::json![false]),
            None,
        )
        .await?;

        debug!("finalizing {:?} took {:?}", self.id, total_start.elapsed());
        Ok(())
    }

    /// Create the default [`AttributePrototypes`](crate::AttributePrototype) and
    /// [`AttributeValues`](crate::AttributeValue) for the [`Props`](Prop) of the
    /// [`SchemaVariant`].
    ///
    /// This method is idempotent, and may be safely called multiple times before
    /// [`SchemaVariant.finalize(ctx)`](SchemaVariant#finalize()) is called.
    pub async fn create_default_prototypes_and_values(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        let root_prop = match Self::find_root_prop(ctx, schema_variant_id).await? {
            Some(root_prop) => root_prop,
            None => return Ok(()),
        };

        Ok(Prop::create_default_prototypes_and_values(ctx, *root_prop.id()).await?)
    }

    /// Creates _internally consuming_ [`InternalProviders`](crate::InternalProvider) corresponding
    /// to every [`Prop`](crate::Prop) in the [`SchemaVariant`] that is not a descendant of an array
    /// or a map.
    async fn create_implicit_internal_providers(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        // If no props have been created for the schema variant, there are no internal providers
        // to create.
        let root_prop = match Self::find_root_prop(ctx, schema_variant_id).await? {
            Some(root_prop) => root_prop,
            None => return Ok(()),
        };

        let mut work_queue = vec![root_prop];

        while let Some(work) = work_queue.pop() {
            let maybe_existing_implicit_internal_provider =
                InternalProvider::find_for_prop(ctx, *work.id()).await?;
            if maybe_existing_implicit_internal_provider.is_none() {
                InternalProvider::new_implicit(ctx, *work.id(), SchemaVariantId::NONE).await?;
            }

            // Only check for child props if the current prop is of kind object.
            if work.kind() == &PropKind::Object {
                let child_props = work.child_props(ctx).await?;
                if !child_props.is_empty() {
                    work_queue.extend(child_props);
                }
            }
        }

        Ok(())
    }

    standard_model_accessor!(ui_hidden, bool, SchemaVariantResult);
    standard_model_accessor!(name, String, SchemaVariantResult);
    standard_model_accessor!(root_prop_id, Option<Pk(PropId)>, SchemaVariantResult);
    standard_model_accessor!(link, Option<String>, SchemaVariantResult);
    standard_model_accessor!(finalized_once, bool, SchemaVariantResult);
    standard_model_accessor!(
        schema_variant_definition_id,
        Option<Pk(SchemaVariantDefinitionId)>,
        SchemaVariantResult
    );

    pub async fn color(&self, ctx: &DalContext) -> SchemaVariantResult<Option<String>> {
        let attribute_value = Component::find_si_child_attribute_value(
            ctx,
            ComponentId::NONE,
            self.id,
            SiPropChild::Color,
        )
        .await
        .map_err(Box::new)?;
        let func_binding_return_value =
            FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
                .await?
                .ok_or_else(|| {
                    SchemaVariantError::FuncBindingReturnValueNotFound(
                        attribute_value.func_binding_return_value_id(),
                    )
                })?;

        let color = func_binding_return_value
            .value()
            .cloned()
            .map(serde_json::from_value)
            .transpose()?;
        Ok(color)
    }

    pub async fn set_color(&self, ctx: &DalContext, color: String) -> SchemaVariantResult<()> {
        let attribute_value = Component::find_si_child_attribute_value(
            ctx,
            ComponentId::NONE,
            self.id,
            SiPropChild::Color,
        )
        .await
        .map_err(Box::new)?;
        let prop = Prop::get_by_id(ctx, &attribute_value.context.prop_id())
            .await?
            .ok_or(PropError::NotFound(
                attribute_value.context.prop_id(),
                *ctx.visibility(),
            ))?;
        prop.set_default_value(ctx, color).await?;
        Ok(())
    }

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "schema_variant_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: SchemaVariantResult,
    );

    standard_model_many_to_many!(
        lookup_fn: sockets,
        associate_fn: add_socket,
        disassociate_fn: remove_socket,
        table_name: "socket_many_to_many_schema_variants",
        left_table: "sockets",
        left_id: SocketId,
        right_table: "schema_variants",
        right_id: SchemaId,
        which_table_is_this: "right",
        returns: Socket,
        result: SchemaVariantResult,
    );

    /// List all direct child [`Props`](crate::Prop) of the [`Prop`](crate::Prop) corresponding
    /// to "/root/si".
    #[instrument(skip_all)]
    pub async fn list_root_si_child_props(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<Prop>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_ROOT_SI_CHILD_PROPS,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    /// Find all [`Props`](crate::Prop) for a given [`SchemaVariantId`](SchemaVariant).
    #[instrument(skip_all)]
    pub async fn all_props(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<Prop>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                ALL_PROPS,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    #[instrument(skip_all)]
    pub async fn list_secret_defining(ctx: &DalContext) -> SchemaVariantResult<Vec<SchemaVariant>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                SECRET_DEFINING_SCHEMA_VARIANTS,
                &[ctx.tenancy(), ctx.visibility()],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    /// Find all [`Func`](crate::Func) objects connected to this schema variant in any way. Only
    /// finds funcs connected at the schema variant context, ignoring any funcs connected to
    /// directly to components. Ignores any functions that have no code (these are typically
    /// intrinsics)
    #[instrument(skip_all)]
    pub async fn all_funcs(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<Func>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                ALL_FUNCS,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;

        Ok(objects_from_rows(rows)?)
    }

    pub async fn upsert_leaf_function(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        component_id: Option<ComponentId>,
        leaf_kind: LeafKind,
        input_locations: &[LeafInputLocation],
        func: &Func,
    ) -> SchemaVariantResult<AttributePrototype> {
        let leaf_prop =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

        let context = match component_id {
            Some(component_id) => AttributeContextBuilder::new()
                .set_prop_id(*leaf_prop.id())
                .set_component_id(component_id)
                .to_context()?,
            None => AttributeContextBuilder::new()
                .set_prop_id(*leaf_prop.id())
                .to_context()?,
        };

        let key = Some(func.name().to_string());
        let mut existing_args = FuncArgument::list_for_func(ctx, *func.id()).await?;
        let mut inputs = vec![];
        for location in input_locations {
            let arg_name = location.arg_name();
            let arg = match existing_args.iter().find(|arg| arg.name() == arg_name) {
                Some(existing_arg) => existing_arg.clone(),
                None => {
                    FuncArgument::new(ctx, arg_name, location.arg_kind(), None, *func.id()).await?
                }
            };

            inputs.push(LeafInput {
                location: *location,
                func_argument_id: *arg.id(),
            });
        }

        for mut existing_arg in existing_args.drain(..) {
            if !inputs.iter().any(
                |&LeafInput {
                     func_argument_id, ..
                 }| func_argument_id == *existing_arg.id(),
            ) {
                existing_arg.delete_by_id(ctx).await?;
            }
        }

        Ok(
            match AttributePrototype::find_for_context_and_key(ctx, context, &key)
                .await?
                .pop()
            {
                Some(existing_proto) => {
                    let mut apas = AttributePrototypeArgument::list_for_attribute_prototype(
                        ctx,
                        *existing_proto.id(),
                    )
                    .await?;

                    for input in &inputs {
                        if !apas
                            .iter()
                            .any(|apa| apa.func_argument_id() == input.func_argument_id)
                        {
                            let input_internal_provider =
                                Self::find_root_child_implicit_internal_provider(
                                    ctx,
                                    schema_variant_id,
                                    input.location.into(),
                                )
                                .await?;

                            AttributePrototypeArgument::new_for_intra_component(
                                ctx,
                                *existing_proto.id(),
                                input.func_argument_id,
                                *input_internal_provider.id(),
                            )
                            .await?;
                        }
                    }

                    for mut apa in apas.drain(..) {
                        if !inputs.iter().any(
                            |&LeafInput {
                                 func_argument_id, ..
                             }| {
                                func_argument_id == apa.func_argument_id()
                            },
                        ) {
                            apa.delete_by_id(ctx).await?;
                        }
                    }

                    existing_proto
                }
                None => {
                    let (_, new_proto) = SchemaVariant::add_leaf(
                        ctx,
                        *func.id(),
                        schema_variant_id,
                        component_id,
                        leaf_kind,
                        inputs,
                    )
                    .await?;

                    new_proto
                }
            },
        )
    }

    /// This method finds all the functions for a particular
    /// ['LeafKind'](crate::schema::variant::leaves::LeafKind) for this SchemaVariant. For example,
    /// it can find all Qualification functions for the variant.
    pub async fn find_leaf_item_functions(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<Vec<(AttributePrototype, Func)>> {
        let leaf_item_prop = Self::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;
        let backend_response_type: FuncBackendResponseType = leaf_kind.into();

        let context = AttributeContextBuilder::new()
            .set_prop_id(*leaf_item_prop.id())
            .to_context()?;

        Ok(
            AttributePrototype::list_prototype_funcs_by_context_and_backend_response_type(
                ctx,
                context,
                backend_response_type,
            )
            .await?,
        )
    }

    /// This method finds a [`leaf`](crate::schema::variant::leaves)'s entry
    /// [`Prop`](crate::Prop) given a [`LeafKind`](crate::schema::variant::leaves::LeafKind).
    pub async fn find_leaf_item_prop(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<Prop> {
        let (leaf_map_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                FIND_LEAF_ITEM_PROP,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &leaf_map_prop_name,
                    &leaf_item_prop_name,
                ],
            )
            .await?;
        Ok(object_from_row(row)?)
    }

    /// Find the implicit [`InternalProvider`](crate::InternalProvider) corresponding to a provided,
    /// [`direct child`](crate::RootPropChild) of [`RootProp`](crate::RootProp).
    pub async fn find_root_child_implicit_internal_provider(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        root_prop_child: RootPropChild,
    ) -> SchemaVariantResult<InternalProvider> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                FIND_ROOT_CHILD_IMPLICIT_INTERNAL_PROVIDER,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &root_prop_child.as_str(),
                ],
            )
            .await?;
        Ok(object_from_row(row)?)
    }

    /// Call [`Self::find_root_prop`] with the [`SchemaVariantId`](SchemaVariant) off
    /// [`self`](SchemaVariant).
    pub async fn root_prop(&self, ctx: &DalContext) -> SchemaVariantResult<Option<Prop>> {
        Self::find_root_prop(ctx, self.id).await
    }

    /// Find the [`Prop`](crate::Prop) corresponding to "/root" for a given
    /// [`SchemaVariantId`](SchemaVariant).
    pub async fn find_root_prop(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Option<Prop>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_ROOT_PROP,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(option_object_from_row(maybe_row)?)
    }

    /// Find the [`SchemaVariant`] for a given [`PropId`](crate::Prop) that resides _anywhere_ in a
    /// [`Prop`](crate::Prop) tree.
    ///
    /// For instance, if you have a [`PropId`](crate::Prop) corresponding to "/root/domain/poop"
    /// and want to know what [`SchemaVariant`]'s [`Prop`](crate::Prop) tree it resides in, use this
    /// method to find out.
    pub async fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> SchemaVariantResult<Option<Self>> {
        // FIXME(nick): this is expensive and should be one query. Please WON'T SOMEBODY THINK OF
        // THE CPU AND THE DATABASE??? OHHHHHHH THE HUMANITY!!!!!!! Oh well, anyway.
        if let Some(root_prop) = Prop::find_root_prop_for_prop(ctx, prop_id).await? {
            for schema_variant in Self::list(ctx).await? {
                if let Some(populated_root_prop_id) = schema_variant.root_prop_id {
                    if *root_prop.id() == populated_root_prop_id {
                        return Ok(Some(schema_variant));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Calls [`Self::find_prop_in_tree`] using the ID off of [`self`](SchemaVariant).
    pub async fn find_prop(&self, ctx: &DalContext, path: &[&str]) -> SchemaVariantResult<Prop> {
        Self::find_prop_in_tree(ctx, self.id, path).await
    }

    /// Find the [`Prop`] in a tree underneath our [`RootProp`] with a given path.
    pub async fn find_prop_in_tree(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        path: &[&str],
    ) -> SchemaVariantResult<Prop> {
        match Prop::find_prop_by_path(ctx, schema_variant_id, &PropPath::new(path)).await {
            Ok(prop) => Ok(prop),
            Err(PropError::NotFoundAtPath(path, visiblity)) => Err(
                SchemaVariantError::PropNotFoundAtPath(schema_variant_id, path, visiblity),
            ),
            Err(err) => Err(err)?,
        }
    }
}
