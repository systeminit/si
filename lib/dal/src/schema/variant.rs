use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::binding::FuncBindingError,
    impl_standard_model, pk,
    schema::{RootProp, SchemaError},
    socket::{Socket, SocketError, SocketId},
    standard_model::{self, objects_from_rows},
    standard_model_accessor, standard_model_belongs_to, standard_model_many_to_many,
    AttributeContextBuilderError, AttributeValueError, DalContext, HistoryEventError, Prop,
    PropError, PropId, Schema, SchemaId, StandardModel, StandardModelError, Timestamp, Visibility,
    WriteTenancy, WsEventError,
};

pub mod root_prop;

#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("schema not found: {0}")]
    NotFound(SchemaVariantId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
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
        ctx: &DalContext<'_, '_>,
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
        let root_prop = RootProp::new(ctx, schema_id, *object.id()).await?;

        object.set_schema(ctx, &schema_id).await?;

        Ok((object, root_prop))
    }

    standard_model_accessor!(name, String, SchemaVariantResult);
    standard_model_accessor!(link, Option<String>, SchemaVariantResult);

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

    pub async fn all_props(&self, ctx: &DalContext<'_, '_>) -> SchemaVariantResult<Vec<Prop>> {
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
}
