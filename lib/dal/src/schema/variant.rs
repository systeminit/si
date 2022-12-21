use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;

use telemetry::prelude::*;
use thiserror::Error;

use crate::edit_field::widget::WidgetKind;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::internal::InternalProviderError;
use crate::standard_model::object_from_row;
use crate::{
    func::binding::FuncBindingError,
    impl_standard_model, pk,
    schema::{RootProp, SchemaError},
    socket::{Socket, SocketError, SocketId},
    standard_model::{self, objects_from_rows},
    standard_model_accessor, standard_model_belongs_to, standard_model_many_to_many,
    AttributeContextBuilderError, AttributePrototypeArgumentError, AttributePrototypeError,
    AttributeValueError, BuiltinsError, DalContext, DiagramKind, ExternalProvider,
    ExternalProviderError, Func, FuncBinding, FuncError, HistoryEventError, InternalProvider, Prop,
    PropError, PropId, PropKind, Schema, SchemaId, SocketArity, StandardModel, StandardModelError,
    Timestamp, Visibility, WriteTenancy, WsEventError,
};
use crate::{FuncBackendResponseType, FuncId};

use self::leaves::LeafKind;

pub mod definition;
pub mod leaves;
pub mod root_prop;

const ALL_PROPS: &str = include_str!("../queries/schema_variant/all_props.sql");
const FIND_LEAF_ITEM_PROP: &str = include_str!("../queries/schema_variant/find_leaf_item_prop.sql");
const FIND_ROOT_CHILD_IMPLICIT_INTERNAL_PROVIDER: &str =
    include_str!("../queries/schema_variant/find_root_child_implicit_internal_provider.sql");

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
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("Schema is missing for SchemaVariant {0}")]
    MissingSchema(SchemaVariantId),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("schema not found: {0}")]
    NotFound(SchemaVariantId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error(transparent)]
    Builtins(#[from] Box<BuiltinsError>),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("std error: {0}")]
    Std(#[from] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("must provide valid schema variant, found unset schema variant id")]
    InvalidSchemaVariant,
    #[error("parent prop not found for prop id: {0}")]
    ParentPropNotFound(PropId),

    // Errors related to definitions.
    #[error("prop not found in cache for name ({0}) and parent prop id ({1})")]
    PropNotFoundInCache(String, PropId),
    #[error("cannot use doc link and doc link ref for prop definition name: ({0})")]
    MultipleDocLinksProvided(String),
    #[error("link not found in doc links map for doc link ref: {0}")]
    LinkNotFoundForDocLinkRef(String),
    #[error("cannot provide entry for object with name: ({0})")]
    FoundEntryForObject(String),
    #[error("must provide children for object with name: ({0})")]
    MissingChildrenForObject(String),
    #[error("cannot provide children for array with name: ({0})")]
    FoundChildrenForArray(String),
    #[error("must provide entry for array with name: ({0})")]
    MissingEntryForArray(String),
    #[error("cannot provide children for primitive with name: ({0})")]
    FoundChildrenForPrimitive(String),
    #[error("cannot provide entry for primitive with name: ({0})")]
    FoundEntryForPrimitive(String),
    #[error("can neither provide children nor entry for primitive with name: ({0})")]
    FoundChildrenAndEntryForPrimitive(String),
    #[error("leaf function response type ({0}) must match leaf kind ({0})")]
    LeafFunctionMismatch(FuncBackendResponseType, LeafKind),
    #[error("leaf function ({0}) must be JsAttribute")]
    LeafFunctionMustBeJsAttribute(FuncId),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

pk!(SchemaVariantPk);
pk!(SchemaVariantId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    name: String,
    link: Option<String>,
    // NOTE(nick): we should consider whether or not we want to keep the color.
    color: Option<i64>,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
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
    // Create a [`SchemaVariant`](Self) with a [`RootProp`](crate::schema::RootProp).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        schema_id: SchemaId,
        name: impl AsRef<str>,
    ) -> SchemaVariantResult<(Self, RootProp)> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM schema_variant_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &name],
            )
            .await?;
        let object: SchemaVariant = standard_model::finish_create_from_row(ctx, row).await?;
        let root_prop = RootProp::new(ctx, *object.id()).await?;

        object.set_schema(ctx, &schema_id).await?;

        // Create frame socket
        let identity_func_name = "si:identity".to_string();
        let identity_func = Func::find_by_attr(ctx, "name", &identity_func_name)
            .await?
            .pop()
            .ok_or(FuncError::NotFoundByName(identity_func_name))?;
        let (identity_func_binding, identity_func_binding_return_value) =
            FuncBinding::create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await?;

        // create a type property for each of the nodes
        // this type will control the functionality of the node
        // the types can be component, or frame types (configuration and aggregation)
        // each schema will set its default type for a node
        let type_prop = Prop::new(
            ctx,
            "type",
            PropKind::String,
            Some((
                WidgetKind::Select,
                Some(serde_json::json!([
                    {
                        "label": "Component",
                        "value": "component",
                    },
                    {
                        "label": "Configuration Frame",
                        "value": "configurationFrame",
                    },
                    {
                        "label": "Aggregation Frame",
                        "value": "aggregationFrame",
                    },
                ])),
            )),
        )
        .await?;
        type_prop.set_parent_prop(ctx, root_prop.si_prop_id).await?;

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
            DiagramKind::Configuration,
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
            DiagramKind::Configuration,
        )
        .await?;

        Ok((object, root_prop))
    }

    /// Calls [`Self::finalize_for_id()`]. Refer to the aforementioned method for more information.
    pub async fn finalize(&self, ctx: &DalContext) -> SchemaVariantResult<()> {
        Self::finalize_for_id(ctx, self.id).await
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
    pub async fn finalize_for_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        Self::create_default_prototypes_and_values(ctx, schema_variant_id).await?;
        Self::create_implicit_internal_providers(ctx, schema_variant_id).await?;
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
        let root_prop = match Prop::find_root_for_schema_variant(ctx, schema_variant_id).await? {
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
        let root_prop = match Prop::find_root_for_schema_variant(ctx, schema_variant_id).await? {
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

    standard_model_accessor!(name, String, SchemaVariantResult);
    standard_model_accessor!(link, Option<String>, SchemaVariantResult);
    standard_model_accessor!(color, OptionBigInt<i64>, SchemaVariantResult);

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

    standard_model_many_to_many!(
        lookup_fn: props,
        associate_fn: add_prop,
        disassociate_fn: remove_prop,
        table_name: "prop_many_to_many_schema_variants",
        left_table: "props",
        left_id: PropId,
        right_table: "schema_variants",
        right_id: SchemaVariantId,
        which_table_is_this: "right",
        returns: Prop,
        result: SchemaVariantResult,
    );

    #[instrument(skip_all)]
    pub async fn all_props(&self, ctx: &DalContext) -> SchemaVariantResult<Vec<Prop>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                ALL_PROPS,
                &[ctx.read_tenancy(), ctx.visibility(), self.id()],
            )
            .await?;
        let results = objects_from_rows(rows)?;
        Ok(results)
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
            .pg()
            .query_one(
                FIND_LEAF_ITEM_PROP,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &leaf_map_prop_name,
                    &leaf_item_prop_name,
                ],
            )
            .await?;
        Ok(object_from_row(row)?)
    }

    /// Finds the implicit [`InternalProvider`](crate::InternalProvider) corresponding to
    /// "/root/domain", which is of kind [`object`](crate::PropKind::Object).
    ///
    /// _Note: the [`SchemaVariant`](crate::SchemaVariant) must be
    /// [`finalized`](crate::SchemaVariant::finalize()) (or
    /// [`finalized for id`](crate::SchemaVariant::finalize_for_id())) before running this query.
    pub async fn find_domain_implicit_internal_provider(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<InternalProvider> {
        Self::find_root_child_implicit_internal_provider(ctx, schema_variant_id, "domain").await
    }

    /// Finds the implicit [`InternalProvider`](crate::InternalProvider) corresponding to
    /// "/root/code", which is of kind [`map`](crate::PropKind::Map).
    ///
    /// _Note: the [`SchemaVariant`](crate::SchemaVariant) must be
    /// [`finalized`](crate::SchemaVariant::finalize()) (or
    /// [`finalized for id`](crate::SchemaVariant::finalize_for_id())) before running this query.
    pub async fn find_code_implicit_internal_provider(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<InternalProvider> {
        Self::find_root_child_implicit_internal_provider(ctx, schema_variant_id, "code").await
    }

    /// Finds the implicit [`InternalProvider`](crate::InternalProvider) corresponding to
    /// "/root/qualification", which is of kind [`map`](crate::PropKind::Map).
    ///
    /// _Note: the [`SchemaVariant`](crate::SchemaVariant) must be
    /// [`finalized`](crate::SchemaVariant::finalize()) (or
    /// [`finalized for id`](crate::SchemaVariant::finalize_for_id())) before running this query.
    pub async fn find_qualification_implicit_internal_provider(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<InternalProvider> {
        Self::find_root_child_implicit_internal_provider(ctx, schema_variant_id, "qualification")
            .await
    }

    /// This private method takes in a provided name for a direct child [`Prop`](crate::Prop)
    /// of "/root" in order to find the child's implicit
    /// [`InternalProvider`](crate::InternalProvider).
    ///
    /// This method is private to ensure compile time safety.
    async fn find_root_child_implicit_internal_provider(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        direct_child_of_root_prop_name: &str,
    ) -> SchemaVariantResult<InternalProvider> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                FIND_ROOT_CHILD_IMPLICIT_INTERNAL_PROVIDER,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &direct_child_of_root_prop_name,
                ],
            )
            .await?;
        Ok(object_from_row(row)?)
    }
}
