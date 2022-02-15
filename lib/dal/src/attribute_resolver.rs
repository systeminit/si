use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{binding::FuncBindingId, binding_return_value::FuncBindingReturnValue, FuncId},
    impl_standard_model, pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, ComponentId, HistoryActor,
    HistoryEventError, IndexMap, Prop, PropError, PropId, PropKind, SchemaId, SchemaVariantId,
    StandardModel, StandardModelError, SystemId, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum AttributeResolverError {
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
    #[error("attribute resolvers must have an associated prop, and this one does not. bug!")]
    MissingProp,
    #[error("attribute resolver not found: {0} ({1:?})")]
    NotFound(AttributeResolverId, Visibility),
    #[error(
        "parent must be for an array, map, or object prop: attribute resolver id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributeResolverId, PropKind),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
}

pub type AttributeResolverResult<T> = Result<T, AttributeResolverError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_FOR_CONTEXT: &str = include_str!("./queries/attribute_resolver_find_for_context.sql");
const FIND_VALUE_FOR_CONTEXT: &str =
    include_str!("./queries/attribute_resolver_find_value_for_context.sql");
const LIST_VALUES_FOR_COMPONENT: &str =
    include_str!("./queries/attribute_resolver_list_values_for_component.sql");

#[derive(Debug)]
pub struct AttributeResolverValue {
    pub prop: Prop,
    pub parent_prop_id: Option<PropId>,
    pub fbrv: FuncBindingReturnValue,
    pub attribute_resolver: AttributeResolver,
    pub parent_attribute_resolver_id: Option<AttributeResolverId>,
}

impl AttributeResolverValue {
    pub fn new(
        prop: Prop,
        parent_prop_id: Option<PropId>,
        fbrv: FuncBindingReturnValue,
        attribute_resolver: AttributeResolver,
        parent_attribute_resolver_id: Option<AttributeResolverId>,
    ) -> Self {
        AttributeResolverValue {
            prop,
            parent_prop_id,
            fbrv,
            attribute_resolver,
            parent_attribute_resolver_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeResolverContext {
    prop_id: PropId,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

impl Default for AttributeResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeResolverContext {
    pub fn new() -> Self {
        AttributeResolverContext {
            prop_id: UNSET_ID_VALUE.into(),
            component_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            system_id: UNSET_ID_VALUE.into(),
        }
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn set_prop_id(&mut self, prop_id: PropId) {
        self.prop_id = prop_id;
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }

    pub fn set_system_id(&mut self, system_id: SystemId) {
        self.system_id = system_id;
    }
}

pk!(AttributeResolverPk);
pk!(AttributeResolverId);

// An AttributeResolver joins a `FuncBinding` to the context in which
// its corresponding `FuncBindingResultValue` is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeResolver {
    pk: AttributeResolverPk,
    id: AttributeResolverId,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    pub index_map: Option<IndexMap>,
    #[serde(flatten)]
    pub context: AttributeResolverContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: AttributeResolver,
    pk: AttributeResolverPk,
    id: AttributeResolverId,
    table_name: "attribute_resolvers",
    history_event_label_base: "attribute_resolver",
    history_event_message_name: "Attribute Resolver"
}

impl AttributeResolver {
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
    ) -> AttributeResolverResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
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
                ],
            )
            .await?;
        let object: AttributeResolver = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        object
            .update_parent_index_map(txn, tenancy, visibility)
            .await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), AttributeResolverResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), AttributeResolverResult);
    standard_model_accessor!(index_map, Option<IndexMap>, AttributeResolverResult);

    standard_model_belongs_to!(
        lookup_fn: parent_attribute_resolver,
        set_fn: set_parent_attribute_resolver_unchecked,
        unset_fn: unset_parent_attribute_resolver,
        table: "attribute_resolver_belongs_to_attribute_resolver",
        model_table: "attribute_resolvers",
        belongs_to_id: AttributeResolverId,
        returns: AttributeResolver,
        result: AttributeResolverResult,
    );

    pub async fn set_parent_attribute_resolver(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        parent_attribute_resolver_id: AttributeResolverId,
    ) -> AttributeResolverResult<()> {
        let parent_attribute_resolver = Self::get_by_id(
            txn,
            self.tenancy(),
            visibility,
            &parent_attribute_resolver_id,
        )
        .await?
        .ok_or_else(|| {
            AttributeResolverError::NotFound(parent_attribute_resolver_id, *visibility)
        })?;
        let parent_prop = Prop::get_by_id(
            txn,
            self.tenancy(),
            visibility,
            &parent_attribute_resolver.context.prop_id,
        )
        .await?
        .ok_or(AttributeResolverError::MissingProp)?;

        match parent_prop.kind() {
            PropKind::Array | PropKind::Map | PropKind::Object => (),
            kind => {
                return Err(AttributeResolverError::ParentNotAllowed(
                    *parent_attribute_resolver.id(),
                    *kind,
                ));
            }
        }

        self.set_parent_attribute_resolver_unchecked(
            txn,
            nats,
            visibility,
            history_actor,
            &parent_attribute_resolver_id,
        )
        .await
    }

    pub fn index_map_mut(&mut self) -> Option<&mut IndexMap> {
        self.index_map.as_mut()
    }

    pub async fn update_stored_index_map(&self, txn: &PgTxn<'_>) -> AttributeResolverResult<()> {
        standard_model::update(
            txn,
            "attribute_resolvers",
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

    #[tracing::instrument(skip(txn))]
    pub async fn update_parent_index_map(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
    ) -> AttributeResolverResult<()> {
        let prop = Prop::get_by_id(txn, tenancy, visibility, &self.context.prop_id())
            .await?
            .ok_or(AttributeResolverError::MissingProp)?;
        if let Some(parent) = prop.parent_prop(txn, visibility).await? {
            match parent.kind() {
                PropKind::Array | PropKind::Map => {
                    let mut parent_context = self.context.clone();
                    parent_context.set_prop_id(*parent.id());
                    if let Some(mut parent_attr_resolver) = AttributeResolver::find_for_context(
                        txn,
                        tenancy,
                        visibility,
                        parent_context,
                    )
                    .await?
                    {
                        match parent_attr_resolver.index_map_mut() {
                            Some(index_map) => {
                                index_map.push(*self.id(), None);
                            }
                            None => {
                                let mut index_map = IndexMap::new();
                                index_map.push(*self.id(), None);
                                parent_attr_resolver.index_map = Some(index_map);
                            }
                        }
                        parent_attr_resolver.update_stored_index_map(txn).await?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(txn))]
    pub async fn find_for_context(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        context: AttributeResolverContext,
    ) -> AttributeResolverResult<Option<Self>> {
        let row = txn
            .query_opt(
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
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn upsert(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: AttributeResolverContext,
    ) -> AttributeResolverResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_resolver_upsert_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
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
        object
            .update_parent_index_map(txn, tenancy, visibility)
            .await?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_value_for_prop_and_component(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        prop_id: PropId,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> AttributeResolverResult<FuncBindingReturnValue> {
        let row = txn
            .query_one(
                FIND_VALUE_FOR_CONTEXT,
                &[&tenancy, &visibility, &prop_id, &component_id, &system_id],
            )
            .await?;
        let object = standard_model::object_from_row(row)?;
        Ok(object)
    }

    /// List all the AttributeResolvers, along with their corresponding prop,
    /// parent prop id, and current value.
    pub async fn list_values_for_component(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> AttributeResolverResult<Vec<AttributeResolverValue>> {
        let rows = txn
            .query(
                LIST_VALUES_FOR_COMPONENT,
                &[&tenancy, &visibility, &component_id, &system_id],
            )
            .await?;
        let mut result = Vec::new();
        for row in rows.into_iter() {
            let fbrv_json: serde_json::Value = row.try_get("object")?;
            let fbrv: FuncBindingReturnValue = serde_json::from_value(fbrv_json)?;
            let prop_json: serde_json::Value = row.try_get("prop_object")?;
            let prop: Prop = serde_json::from_value(prop_json)?;
            let parent_prop_id: Option<PropId> = row.try_get("parent_prop_id")?;
            let attribute_resolver_json: serde_json::Value =
                row.try_get("attribute_resolver_object")?;
            let attribute_resolver: AttributeResolver =
                serde_json::from_value(attribute_resolver_json)?;
            let parent_attribute_resolver_id: Option<AttributeResolverId> =
                row.try_get("parent_attribute_resolver_id")?;
            result.push(AttributeResolverValue::new(
                prop,
                parent_prop_id,
                fbrv,
                attribute_resolver,
                parent_attribute_resolver_id,
            ));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::AttributeResolverContext;

    #[test]
    fn context_builder() {
        let mut c = AttributeResolverContext::new();
        c.set_component_id(15.into());
        c.set_prop_id(22.into());
        assert_eq!(c.component_id(), 15.into());
        assert_eq!(c.prop_id(), 22.into());
    }
}
