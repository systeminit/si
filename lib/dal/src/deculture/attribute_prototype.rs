use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    deculture::{
        attribute_resolver_context::AttributeResolverContext,
        attribute_resolver_context::AttributeResolverContextError,
        attribute_value::{AttributeValue, AttributeValueError, AttributeValueId},
    },
    func::binding::{FuncBindingError, FuncBindingId},
    func::FuncId,
    impl_standard_model, pk,
    standard_model::{self},
    standard_model_accessor, HistoryActor, HistoryEventError, PropError, PropKind, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
};

const LIST_FOR_CONTEXT: &str = include_str!("./queries/attribute_prototype_list_for_context.sql");
const FIND_WITH_PARENT_VALUE_AND_KEY_FOR_CONTEXT: &str =
    include_str!("./queries/attribute_prototype_find_with_parent_value_and_key_for_context.sql");

#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("attribute resolver context builder error: {0}")]
    AttributeResolverContextBuilder(#[from] AttributeResolverContextError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
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
    #[error("missing attribute value for tenancy {0:?}, visibility {1:?}, prototype {2:?}, with parent attribute value {3:?}")]
    MissingValue(
        Tenancy,
        Visibility,
        AttributePrototypeId,
        Option<AttributeValueId>,
    ),
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
        parent_attribute_value_id: Option<AttributeValueId>,
    ) -> AttributePrototypeResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_prototype_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &tenancy,
                    &visibility,
                    &context,
                    &func_id,
                    &func_binding_id,
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

        let value = AttributeValue::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            None,
            context,
            key.clone(),
        )
        .await?;

        if let Some(parent_attribute_value_id) = parent_attribute_value_id {
            value
                .set_parent_attribute_value(
                    txn,
                    nats,
                    visibility,
                    history_actor,
                    &parent_attribute_value_id,
                )
                .await?;
        }

        if !context.is_least_specific() {
            let original_prototype = Self::find_with_parent_value_and_key_for_context(
                txn,
                tenancy,
                visibility,
                parent_attribute_value_id,
                key,
                context.less_specific()?,
            )
            .await?;

            if let Some(original_prototype) = original_prototype {
                Self::create_intermediate_proxy_values(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    parent_attribute_value_id,
                    *original_prototype.id(),
                    context.less_specific()?,
                )
                .await?;
            }
        }

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
                LIST_FOR_CONTEXT,
                &[&tenancy, &visibility, &context, &context.prop_id()],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }

    #[tracing::instrument(skip(txn))]
    pub async fn find_with_parent_value_and_key_for_context(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        parent_attribute_value_id: Option<AttributeValueId>,
        key: Option<String>,
        context: AttributeResolverContext,
    ) -> AttributePrototypeResult<Option<Self>> {
        let row = txn
            .query_opt(
                FIND_WITH_PARENT_VALUE_AND_KEY_FOR_CONTEXT,
                &[
                    &tenancy,
                    &visibility,
                    &context,
                    &parent_attribute_value_id,
                    &key,
                ],
            )
            .await?;

        Ok(standard_model::option_object_from_row(row)?)
    }

    #[tracing::instrument(skip(txn))]
    #[allow(clippy::too_many_arguments)]
    #[async_recursion]
    async fn create_intermediate_proxy_values(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        parent_attribute_value_id: Option<AttributeValueId>,
        prototype_id: AttributePrototypeId,
        context: AttributeResolverContext,
    ) -> AttributePrototypeResult<()> {
        if context.is_least_specific() {
            return Ok(());
        }

        if (AttributeValue::find_with_parent_and_prototype_for_context(
            txn,
            tenancy,
            visibility,
            parent_attribute_value_id,
            prototype_id,
            context,
        )
        .await?)
            .is_none()
        {
            // Need to create a proxy to the next lowest level
            Self::create_intermediate_proxy_values(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                parent_attribute_value_id,
                prototype_id,
                context.less_specific()?,
            )
            .await?;

            if let Some(proxy_target) = AttributeValue::find_with_parent_and_prototype_for_context(
                txn,
                tenancy,
                visibility,
                parent_attribute_value_id,
                prototype_id,
                context.less_specific()?,
            )
            .await?
            {
                // Create the proxy at this level
                let mut proxy_attribute_value = AttributeValue::new(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    proxy_target.func_binding_return_value_id().copied(),
                    context,
                    proxy_target.key().map(|k| k.to_string()),
                )
                .await?;
                proxy_attribute_value
                    .set_proxy_for_attribute_value_id(
                        txn,
                        nats,
                        visibility,
                        history_actor,
                        Some(*proxy_target.id()),
                    )
                    .await?;
                proxy_attribute_value
                    .set_attribute_prototype(txn, nats, visibility, history_actor, &prototype_id)
                    .await?
            } else {
                return Err(AttributePrototypeError::MissingValue(
                    tenancy.clone(),
                    *visibility,
                    prototype_id,
                    parent_attribute_value_id,
                ));
            }
        }

        Ok(())
    }
}
