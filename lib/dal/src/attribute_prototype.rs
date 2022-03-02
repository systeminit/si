use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute_resolver_context::AttributeResolverContext,
    attribute_value::AttributeValue,
    func::{
        binding::{FuncBindingError, FuncBindingId},
        binding_return_value::FuncBindingReturnValueError,
    },
    func::{binding_return_value::FuncBindingReturnValue, FuncId},
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    HistoryActor, HistoryEventError, Prop, PropError, PropKind, StandardModel, StandardModelError,
    Tenancy, Timestamp, Visibility,
};

const FIND_FOR_CONTEXT: &str = include_str!("./queries/attribute_prototype_list_for_context.sql");

#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
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

        let fbrv = FuncBindingReturnValue::get_by_func_binding_id(
            txn,
            tenancy,
            visibility,
            func_binding_id,
        )
        .await?
        .map(|fbrv| *fbrv.id());

        let attribute_value = AttributeValue::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            fbrv,
            context,
            key,
        )
        .await
        .map_err(|e| AttributePrototypeError::AttributeValue(format!("{e}")))?;
        attribute_value
            .set_attribute_prototype(txn, nats, visibility, history_actor, object.id())
            .await
            .map_err(|e| AttributePrototypeError::AttributeValue(format!("{e}")))?;

        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), AttributePrototypeResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), AttributePrototypeResult);
    standard_model_accessor!(key, Option<String>, AttributePrototypeResult);

    standard_model_belongs_to!(
        lookup_fn: parent_attribute_prototype,
        set_fn: set_parent_attribute_prototype_unchecked,
        unset_fn: unset_parent_attribute_prototype,
        table: "attribute_prototype_belongs_to_attribute_prototype",
        model_table: "attribute_prototypes",
        belongs_to_id: AttributePrototypeId,
        returns: AttributePrototype,
        result: AttributePrototypeResult,
    );

    pub async fn set_parent_attribute_prototype(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        parent_attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<()> {
        let parent_attribute_prototype = Self::get_by_id(
            txn,
            self.tenancy(),
            visibility,
            &parent_attribute_prototype_id,
        )
        .await?
        .ok_or(AttributePrototypeError::NotFound(
            parent_attribute_prototype_id,
            *visibility,
        ))?;
        let parent_prop = Prop::get_by_id(
            txn,
            self.tenancy(),
            visibility,
            &parent_attribute_prototype.context.prop_id(),
        )
        .await?
        .ok_or(AttributePrototypeError::MissingProp)?;

        match parent_prop.kind() {
            PropKind::Array | PropKind::Map | PropKind::Object => (),
            kind => {
                return Err(AttributePrototypeError::ParentNotAllowed(
                    *parent_attribute_prototype.id(),
                    *kind,
                ));
            }
        }

        self.set_parent_attribute_prototype_unchecked(
            txn,
            nats,
            visibility,
            history_actor,
            &parent_attribute_prototype_id,
        )
        .await
    }

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
