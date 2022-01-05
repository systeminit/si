use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::node::NodeKind;
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    HistoryActor, HistoryEventError, Node, NodeError, Schema, SchemaError, SchemaId, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("node error: {0}")]
    NodeError(#[from] NodeError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

pk!(ComponentPk);
pk!(ComponentId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pk: ComponentPk,
    id: ComponentId,
    name: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Component,
    pk: ComponentPk,
    id: ComponentId,
    table_name: "components",
    history_event_label_base: "component",
    history_event_message_name: "Component"
}

impl Component {
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> ComponentResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new_for_schema_variant_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let name = name.as_ref();

        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;
        let schema_variant =
            SchemaVariant::get_by_id(txn, &schema_tenancy, visibility, schema_variant_id)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = schema_variant
            .schema(txn, visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let row = txn
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &name],
            )
            .await?;

        let component: Component = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        component
            .set_schema(txn, nats, visibility, history_actor, schema.id())
            .await?;
        component
            .set_schema_variant(txn, nats, visibility, history_actor, schema_variant.id())
            .await?;

        let node = Node::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &NodeKind::Component,
        )
        .await?;
        node.set_component(txn, nats, visibility, history_actor, component.id())
            .await?;

        Ok((component, node))
    }

    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new_application_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> ComponentResult<(Self, Node)> {
        let universal_tenancy = Tenancy::new_universal();

        let schema_variant_id = Schema::default_schema_variant_id_for_name(
            txn,
            &universal_tenancy,
            visibility,
            "application",
        )
        .await?;

        let (component, node) = Component::new_for_schema_variant_with_node(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            name,
            &schema_variant_id,
        )
        .await?;
        Ok((component, node))
    }

    standard_model_accessor!(name, String, ComponentResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "component_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: ComponentResult,
    );

    standard_model_belongs_to!(
        lookup_fn: schema_variant,
        set_fn: set_schema_variant,
        unset_fn: unset_schema_variant,
        table: "component_belongs_to_schema_variant",
        model_table: "schema_variants",
        belongs_to_id: SchemaVariantId,
        returns: SchemaVariant,
        result: ComponentResult,
    );
}
