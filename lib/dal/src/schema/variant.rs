use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::internal::InternalProviderError;
use crate::{
    func::binding::FuncBindingError,
    impl_standard_model, pk,
    schema::{RootProp, SchemaError},
    socket::{Socket, SocketError, SocketId},
    standard_model::{self, objects_from_rows},
    standard_model_accessor, standard_model_belongs_to, standard_model_many_to_many,
    AttributeContextBuilderError, AttributePrototypeError, AttributeValueError, DalContext,
    HistoryEventError, InternalProvider, Prop, PropError, PropId, PropKind, Schema, SchemaId,
    StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy, WsEventError,
};

pub mod root_prop;

#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
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
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("root prop not found for schema variant: {0}")]
    RootPropNotFound(SchemaVariantId),
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
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

const ALL_PROPS: &str = include_str!("../queries/schema_variant_all_props.sql");

pk!(SchemaVariantPk);
pk!(SchemaVariantId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    name: String,
    link: Option<String>,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_, '_>,
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

        Ok((object, root_prop))
    }

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
    /// [`SchemaVariant`]. This method should only be called once, as it is not fully idempotent.
    pub async fn finalize(&self, ctx: &DalContext<'_, '_, '_>) -> SchemaVariantResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(SchemaVariantError::MissingSchema(self.id))?;

        Self::create_default_prototypes_and_values(ctx, self.id).await?;
        Self::create_implicit_internal_providers(ctx, *schema.id(), self.id).await?;

        Ok(())
    }

    /// Create the default [`AttributePrototypes`](crate::AttributePrototype) and
    /// [`AttributeValues`](crate::AttributeValue) for the [`Props`](Prop) of the
    /// [`SchemaVariant`].
    ///
    /// This method is idempotent, and may be safely called multiple times before
    /// [`SchemaVariant.finalize(ctx)`](SchemaVariant#finalize()) is called.
    pub async fn create_default_prototypes_and_values(
        ctx: &DalContext<'_, '_, '_>,
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
        ctx: &DalContext<'_, '_, '_>,
        schema_id: SchemaId,
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
            InternalProvider::new_implicit(ctx, *work.id(), schema_id, schema_variant_id).await?;

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
    pub async fn all_props(&self, ctx: &DalContext<'_, '_, '_>) -> SchemaVariantResult<Vec<Prop>> {
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

    /// Find the root [`Prop`](crate::Prop) for [`Self`](Self).
    pub async fn root_prop(&self, ctx: &DalContext<'_, '_, '_>) -> SchemaVariantResult<Prop> {
        // FIXME(nick): this is an inefficient solution that would be better suited by a database query.
        for prop in self.all_props(ctx).await? {
            if prop.parent_prop(ctx).await?.is_none() && prop.name() == "root" {
                return Ok(prop);
            }
        }
        return Err(SchemaVariantError::RootPropNotFound(*self.id()));
    }
}
