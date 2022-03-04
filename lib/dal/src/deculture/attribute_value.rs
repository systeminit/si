use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};

use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    deculture::{
        attribute_prototype::{AttributePrototype, AttributePrototypeId},
        attribute_resolver_context::AttributeResolverContext,
    },
    func::{binding::FuncBindingError, binding_return_value::FuncBindingReturnValueId},
    impl_standard_model, pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, HistoryActor, HistoryEventError, IndexMap,
    PropError, PropKind, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};

// const FIND_FOR_CONTEXT_WITH_PARENT: &str = include_str!("../queries/attribute_value_find_for_context_with_parent.sql");

#[derive(Error, Debug)]
pub enum AttributeValueError {
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
    #[error(
        "attribute values must have an associated attribute prototype, and this one does not. bug!"
    )]
    MissingAttributePrototype,
    #[error("attribute value not found: {0} ({1:?})")]
    NotFound(AttributeValueId, Visibility),
    #[error(
        "parent must be for an array, map, or object prop: attribute resolver id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributeValueId, PropKind),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type AttributeValueResult<T> = Result<T, AttributeValueError>;

pk!(AttributeValuePk);
pk!(AttributeValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    pk: AttributeValuePk,
    id: AttributeValueId,
    /// The [`FuncBindingReturnValueId`] that represents the value at this specific position & context.
    /// A [`None`] value here represents that that the [`Func`](crate::Func) in the associated
    /// [`AttributePrototype`] has not yet generated a [`FuncBindingReturnValueId`] for its
    /// [`FuncBinding`](crate::func::binding::FuncBinding).
    func_binding_return_value_id: Option<FuncBindingReturnValueId>,
    /// The [`AttributeValueId`] (from a less-specific [`AttributeResolverContext`]) that this
    /// [`AttributeValue`] is standing in for in this more-specific [`AttributeResolverContext`].
    proxy_for_attribute_value_id: Option<AttributeValueId>,
    /// If this is a `sealed_proxy`, then it should **not** update its [`FuncBindingReturnValueId`] from the
    /// [`AttributeValue`] referenced to in `proxy_for_attribute_value_id`.
    sealed_proxy: bool,
    pub index_map: Option<IndexMap>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub context: AttributeResolverContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl_standard_model! {
    model: AttributeValue,
    pk: AttributeValuePk,
    id: AttributeValueId,
    table_name: "attribute_values",
    history_event_label_base: "attribute_value",
    history_event_message_name: "Attribute Value"
}

impl AttributeValue {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        func_binding_return_value_id: Option<FuncBindingReturnValueId>,
        context: AttributeResolverContext,
        key: Option<String>,
    ) -> AttributeValueResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_value_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &tenancy,
                    &visibility,
                    &func_binding_return_value_id,
                    &context.prop_id(),
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                    &key,
                ],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        // TODO: We need to have proxies for values from "less specific" contexts before we can handle updating our parent's index_map.
        //
        // object
        //     .update_parent_index_map(txn, tenancy, visibility)
        //     .await?;

        // for a prop in an object in an array...
        // important: if anything in grandparent to root is a map or an array, you need to know which attribute value is your parent
        // (there could be multiple)
        //
        // if you are in a component, the root will never be map or array, but free floating prop could be map or array
        //
        // why: if a parent is a map or an array, the value is just a value or an array, but
        // in the other scenario, you need to know which element or map value you are a child of.
        //
        // {
        //
        //
        //
        //

        // let mut
        // let contexts = vec![context];
        // context.less_specific()

        Ok(object)
    }

    standard_model_accessor!(
        proxy_for_attribute_value_id,
        OptionBigInt<AttributeValueId>,
        AttributeValueResult
    );
    standard_model_accessor!(sealed_proxy, bool, AttributeValueResult);
    standard_model_accessor!(
        func_binding_return_value_id,
        OptionBigInt<FuncBindingReturnValueId>,
        AttributeValueResult
    );
    standard_model_accessor!(index_map, Option<IndexMap>, AttributeValueResult);
    standard_model_accessor!(key, Option<String>, AttributeValueResult);

    standard_model_belongs_to!(
        lookup_fn: parent_attribute_value,
        set_fn: set_parent_attribute_value,
        unset_fn: unset_parent_attribute_value,
        table: "attribute_value_belongs_to_attribute_value",
        model_table: "attribute_values",
        belongs_to_id: AttributeValueId,
        returns: AttributeValue,
        result: AttributeValueResult,
    );

    standard_model_belongs_to!(
        lookup_fn: attribute_prototype,
        set_fn: set_attribute_prototype,
        unset_fn: unset_attribute_prototype,
        table: "attribute_value_belongs_to_attribute_prototype",
        model_table: "attribute_prototypes",
        belongs_to_id: AttributePrototypeId,
        returns: AttributePrototype,
        result: AttributeValueResult,
    );

    pub fn index_map_mut(&mut self) -> Option<&mut IndexMap> {
        self.index_map.as_mut()
    }

    pub async fn update_stored_index_map(&self, txn: &PgTxn<'_>) -> AttributeValueResult<()> {
        standard_model::update(
            txn,
            "attribute_values",
            "index_map",
            self.tenancy(),
            self.visibility(),
            self.id(),
            &self.index_map,
            TypeHint::JsonB,
        )
        .await?;
        Ok(())
    }

    // We need...
    // - context
    // - prototype (need elements in array / values in map)
    // - parent (AttributeValue)
    //   - if None, must not have parent in same visibility, tenancy, etc.
    //   - if Some, must have same parent
    pub async fn find() {}

    // pub async fn update_proxies(
    //     &mut self,
    //     txn: &PgTxn<'_>,
    //     nats: &NatsTxn,
    //     history_actor: &HistoryActor,
    // ) -> AttributeValueResult<()> {
    //     let proxied_attribute_value_id = match self.proxy_for_attribute_value_id() {
    //         Some(id) => id,
    //         None => return Ok(()),
    //     };
    //     if self.sealed_proxy() {
    //         return Ok(());
    //     }

    //     let proxied_attribute_value = Self::get_by_id(
    //         txn,
    //         self.tenancy(),
    //         self.visibility(),
    //         proxied_attribute_value_id,
    //     )
    //     .await?
    //     .ok_or(AttributeValueError::NotFound(
    //         *proxied_attribute_value_id,
    //         *self.visibility(),
    //     ))?;
    //     if proxied_attribute_value.key() != self.key() {
    //         // The far side of the proxy changed its key, so we need to stop considering *this* a valid proxy
    //         // for it, and potentially create a new one, by removing this (and all child proxies), and asking
    //         // our parent AttributeValue to refresh itself. If we're updating things Root -> Leaf, we
    //         // probably don't need to do this, though, as both of the above should already be handled by the
    //         // time we get to this node.
    //     }

    //     // TODO: We'll want to create new proxies for values under the proxied_attribute_value, if we're
    //     //       proxying an Array/Hash/Map, and remove proxies for values that no longer exist.

    //     // TODO: All of the "update the proxy" logic is probably best handled from the source side of the
    //     //       proxy, and asking it to propagate its changes out to the things proxying it.

    //     let our_visibility = self.visibility.clone();
    //     self.set_func_binding_return_value_id(
    //         txn,
    //         nats,
    //         &our_visibility,
    //         history_actor,
    //         proxied_attribute_value.func_binding_return_value_id(),
    //     )
    //     .await?;

    //     Ok(())
    // }
}
