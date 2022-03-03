use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute_resolver_context::AttributeResolverContext,
    func::binding::{FuncBindingError, FuncBindingId},
    func::FuncId,
    impl_standard_model, pk,
    standard_model::{self},
    standard_model_accessor, HistoryActor, HistoryEventError, PropError, PropKind, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
};

const FIND_FOR_CONTEXT: &str = include_str!("./queries/attribute_prototype_list_for_context.sql");

#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("func not found: {0}")]
    MissingFunc(String),
    #[error("attribute prototypes must have an associated prop, and this one does not. bug!")]
    MissingProp,
    #[error("attribute prototype not found: {0} ({1:?})")]
    NotFound(AttributePrototypeId, Visibility),
    #[error(
        "parent must be for an array, map, or object prop: attribute prototype id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributePrototypeId, PropKind),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type AttributePrototypeResult<T> = Result<T, AttributePrototypeError>;

pk!(AttributePrototypePk);
pk!(AttributePrototypeId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototype {
    pk: AttributePrototypePk,
    id: AttributePrototypeId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    visibility: Visibility,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    pub key: Option<String>,
    #[serde(flatten)]
    pub context: AttributeResolverContext,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl_standard_model! {
    model: AttributePrototype,
    pk: AttributePrototypePk,
    id: AttributePrototypeId,
    table_name: "attribute_prototypes",
    history_event_label_base: "attribute_prototype",
    history_event_message_name: "Attribute Prototype"
}

impl AttributePrototype {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: AttributeResolverContext,
        key: Option<String>,
    ) -> AttributePrototypeResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &tenancy,
                    &visibility,
                    &func_id,
                    &func_binding_id,
                    &context.prop_id(),
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                    &key,
                ],
            )
            .await?;
        let object: AttributePrototype = standard_model::finish_create_from_row(
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

    standard_model_accessor!(func_id, Pk(FuncId), AttributePrototypeResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), AttributePrototypeResult);
    standard_model_accessor!(key, Option<String>, AttributePrototypeResult);

    #[tracing::instrument(skip(txn))]
    pub async fn list_for_context(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        context: AttributeResolverContext,
    ) -> AttributePrototypeResult<Vec<Self>> {
        let rows = txn
            .query(
                FIND_FOR_CONTEXT,
                &[
                    &tenancy,
                    &visibility,
                    &context.prop_id(),
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }
}
