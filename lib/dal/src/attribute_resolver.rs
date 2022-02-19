use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{
        backend::{
            array::FuncBackendArrayArgs, boolean::FuncBackendBooleanArgs,
            integer::FuncBackendIntegerArgs, map::FuncBackendMapArgs,
            prop_object::FuncBackendPropObjectArgs, string::FuncBackendStringArgs,
        },
        binding::{FuncBinding, FuncBindingError, FuncBindingId},
        binding_return_value::FuncBindingReturnValue,
        FuncId,
    },
    impl_standard_model, pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, ComponentId, Func, FuncBackendKind,
    HistoryActor, HistoryEventError, IndexMap, Prop, PropError, PropId, PropKind, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, SystemId, Tenancy, Timestamp, Visibility,
};
use async_recursion::async_recursion;

#[derive(Error, Debug)]
pub enum AttributeResolverError {
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
    #[error("attribute resolvers must have an associated prop, and this one does not. bug!")]
    MissingProp,
    #[error("attribute resolver not found: {0} ({1:?})")]
    NotFound(AttributeResolverId, Visibility),
    #[error(
        "parent must be for an array, map, or object prop: attribute resolver id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributeResolverId, PropKind),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type AttributeResolverResult<T> = Result<T, AttributeResolverError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_FOR_CONTEXT: &str = include_str!("./queries/attribute_resolver_find_for_context.sql");
const FIND_VALUE_FOR_CONTEXT: &str =
    include_str!("./queries/attribute_resolver_find_value_for_context.sql");
const LIST_VALUES_FOR_COMPONENT: &str =
    include_str!("./queries/attribute_resolver_list_values_for_component.sql");
const SIBLINGS_HAVE_SET_VALUES: &str =
    include_str!("./queries/attribute_resolver_siblings_have_set_values.sql");

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
    pub key: Option<String>,
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
        key: Option<String>,
    ) -> AttributeResolverResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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
    standard_model_accessor!(key, Option<String>, AttributeResolverResult);

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
                                index_map.push(*self.id(), self.key.clone());
                            }
                            None => {
                                let mut index_map = IndexMap::new();
                                index_map.push(*self.id(), self.key.clone());
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
        key: Option<String>,
    ) -> AttributeResolverResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM attribute_resolver_upsert_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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

    /// Check if there are any [`AttributeResolver`]s that are also children of the provided
    /// `AttributeResolverId`'s parent (siblings of the provided `AttributeResolverId`) that have a value
    /// other than `FuncBackendKind::Unset`.
    pub async fn any_siblings_are_set(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        attribute_resolver_id: AttributeResolverId,
    ) -> AttributeResolverResult<bool> {
        let row = txn
            .query_one(
                SIBLINGS_HAVE_SET_VALUES,
                &[&tenancy, &visibility, &attribute_resolver_id],
            )
            .await?;
        let siblings_have_values: bool = row.try_get("siblings_are_set")?;

        Ok(siblings_have_values)
    }

    /// Update the [`Func`] & [`FuncBinding`] of an [`AttributeResolver`] for a given
    /// [`AttributeResolverContext`]. If the [`AttributeResolver`] exists, but is not specific to the given
    /// [`AttributeResolverContext`], then a new [`AttributeResolver`] is created, specific to that
    /// [`AttributeResolverContext`].
    #[allow(clippy::too_many_arguments)]
    #[async_recursion]
    pub async fn update_for_context(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        attribute_resolver_id: AttributeResolverId,
        attribute_resolver_context: AttributeResolverContext,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeResolverResult<(Option<serde_json::Value>, AttributeResolverId)> {
        // Find the attribute resolver to update.
        let given_attribute_resolver =
            Self::get_by_id(txn, tenancy, visibility, &attribute_resolver_id)
                .await?
                .ok_or_else(|| {
                    AttributeResolverError::NotFound(attribute_resolver_id, *visibility)
                })?;

        // If the context isn't the _specific_ context that we're trying to update, make a new one.
        // This is necessary, since the one that we were given might be the "default" one that is directly
        // attached to a Prop in a SchemaVariant, and the AttributeResolverContext might be specifying that
        // we want to have a value for a specific Component/System.
        let mut attribute_resolver =
            if given_attribute_resolver.context == attribute_resolver_context {
                given_attribute_resolver
            } else {
                Self::new(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    given_attribute_resolver.func_id(),
                    given_attribute_resolver.func_binding_id(),
                    attribute_resolver_context.clone(),
                    given_attribute_resolver.key,
                )
                .await?
            };

        let prop = Prop::get_by_id(
            txn,
            tenancy,
            visibility,
            &attribute_resolver.context.prop_id(),
        )
        .await?
        .ok_or(AttributeResolverError::MissingProp)?;

        let (func_name, func_args) = match (prop.kind(), value.clone()) {
            (_, None) => ("si:unset", serde_json::to_value(())?),
            (PropKind::Array, Some(_)) => {
                let value: Vec<serde_json::Value> = as_type(serde_json::json![[]])?;
                (
                    "si:setArray",
                    serde_json::to_value(FuncBackendArrayArgs::new(value))?,
                )
            }
            (PropKind::Boolean, Some(value_json)) => {
                let value: bool = as_type(value_json)?;
                (
                    "si:setBoolean",
                    serde_json::to_value(FuncBackendBooleanArgs::new(value))?,
                )
            }
            (PropKind::Integer, Some(value_json)) => {
                let value: i64 = as_type(value_json)?;
                (
                    "si:setInteger",
                    serde_json::to_value(FuncBackendIntegerArgs::new(value))?,
                )
            }
            (PropKind::Map, Some(_)) => {
                let value: serde_json::Map<String, serde_json::Value> =
                    as_type(serde_json::json![{}])?;
                (
                    "si:setMap",
                    serde_json::to_value(FuncBackendMapArgs::new(value))?,
                )
            }
            (PropKind::Object, Some(_)) => {
                let value: serde_json::Map<String, serde_json::Value> =
                    as_type(serde_json::json![{}])?;
                (
                    "si:setPropObject",
                    serde_json::to_value(FuncBackendPropObjectArgs::new(value))?,
                )
            }
            (PropKind::String, Some(value_json)) => {
                let value: String = as_type(value_json)?;
                (
                    "si:setString",
                    serde_json::to_value(FuncBackendStringArgs::new(value))?,
                )
            }
        };

        let (func, func_binding, _) = set_value(
            txn,
            nats,
            veritech.clone(),
            tenancy,
            visibility,
            history_actor,
            func_name,
            func_args,
        )
        .await?;

        attribute_resolver
            .set_func_id(txn, nats, visibility, history_actor, *func.id())
            .await?;
        attribute_resolver
            .set_func_binding_id(txn, nats, visibility, history_actor, *func_binding.id())
            .await?;
        attribute_resolver
            .set_key(txn, nats, visibility, history_actor, key)
            .await?;

        if let (Some(parent_prop), Some(parent_attribute_resolver)) = (
            prop.parent_prop(txn, visibility).await?,
            attribute_resolver
                .parent_attribute_resolver(txn, visibility)
                .await?,
        ) {
            let current_parent_value = Self::find_value_for_prop_and_component(
                txn,
                tenancy,
                visibility,
                *parent_prop.id(),
                attribute_resolver_context.component_id(),
                attribute_resolver_context.system_id(),
            )
            .await?;

            let mut parent_attribute_resolver_context = attribute_resolver_context.clone();
            parent_attribute_resolver_context.set_prop_id(*parent_prop.id());

            let (should_update_parent, new_parent_value) = if value.is_some() {
                (true, Some(serde_json::to_value(())?))
            } else if Self::any_siblings_are_set(txn, tenancy, visibility, *attribute_resolver.id())
                .await?
            {
                (false, None)
            } else {
                (true, None)
            };

            if should_update_parent
                && current_parent_value.value().is_some() != new_parent_value.is_some()
            {
                let (_, parent_attribute_resolver_id) = Self::update_for_context(
                    txn,
                    nats,
                    veritech,
                    tenancy,
                    visibility,
                    history_actor,
                    *parent_attribute_resolver.id(),
                    parent_attribute_resolver_context,
                    new_parent_value,
                    parent_attribute_resolver.key().map(|s| s.to_string()),
                )
                .await?;

                // We need to set our parent AttributeResolverId *after* potentially auto-vivifying our parent,
                // because we need to use whatever the AttributeResolverId is for our parent after it's been set to
                // whatever its "final" value is going to be in our context.
                attribute_resolver
                    .set_parent_attribute_resolver(
                        txn,
                        nats,
                        visibility,
                        history_actor,
                        parent_attribute_resolver_id,
                    )
                    .await?
            }
        };

        Ok((value, *attribute_resolver.id()))
    }

    /// Insert a new value for [`Prop`] in the given [`AttributeResolverContext`]. This is mostly only useful for
    /// adding elements to a [`PropKind::Array`], or [`PropKind::Map`]. All other [`PropKind`] should be able to
    /// directly use [`update_for_context()`](AttributeResolver::update_for_context()), as there will already be an
    /// appropriate [`AttributeResolver`] to use.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_for_context(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        attribute_resolver_context: AttributeResolverContext,
        parent_attribute_resolver_id: AttributeResolverId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeResolverResult<(Option<serde_json::Value>, AttributeResolverId)> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let parent_attribute_resolver =
            Self::get_by_id(txn, tenancy, visibility, &parent_attribute_resolver_id)
                .await?
                .ok_or(AttributeResolverError::NotFound(
                    parent_attribute_resolver_id,
                    *visibility,
                ))?;

        let mut parent_attribute_resolver_context = attribute_resolver_context.clone();
        parent_attribute_resolver_context.set_prop_id(parent_attribute_resolver.context.prop_id());
        let (_, populated_parent_attribute_resolver_id) = Self::update_for_context(
            txn,
            nats,
            veritech.clone(),
            tenancy,
            visibility,
            history_actor,
            parent_attribute_resolver_id,
            parent_attribute_resolver_context,
            Some(serde_json::json![()]),
            parent_attribute_resolver.key,
        )
        .await?;

        let unset_func_name = "si:unset".to_string();
        let unset_func =
            Func::find_by_attr(txn, &schema_tenancy, visibility, "name", &unset_func_name)
                .await?
                .pop()
                .ok_or(AttributeResolverError::MissingFunc(unset_func_name))?;
        let (unset_func_binding, _) = FuncBinding::find_or_create(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            serde_json::json![null],
            *unset_func.id(),
            FuncBackendKind::Unset,
        )
        .await?;

        let attribute_resolver = Self::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            *unset_func.id(),
            *unset_func_binding.id(),
            attribute_resolver_context.clone(),
            key.clone(),
        )
        .await?;
        attribute_resolver
            .set_parent_attribute_resolver(
                txn,
                nats,
                visibility,
                history_actor,
                populated_parent_attribute_resolver_id,
            )
            .await?;

        Ok(Self::update_for_context(
            txn,
            nats,
            veritech,
            tenancy,
            visibility,
            history_actor,
            *attribute_resolver.id(),
            attribute_resolver_context,
            value,
            key,
        )
        .await?)
    }
}

fn as_type<T: serde::de::DeserializeOwned>(json: serde_json::Value) -> AttributeResolverResult<T> {
    T::deserialize(&json).map_err(|_| {
        AttributeResolverError::InvalidPropValue(std::any::type_name::<T>().to_owned(), json)
    })
}

#[allow(clippy::too_many_arguments)]
async fn set_value(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    func_name: &str,
    args: serde_json::Value,
) -> AttributeResolverResult<(Func, FuncBinding, bool)> {
    let mut schema_tenancy = tenancy.clone();
    schema_tenancy.universal = true;

    let func_name = func_name.to_owned();
    let func = Func::find_by_attr(txn, &schema_tenancy, visibility, "name", &func_name)
        .await?
        .pop()
        .ok_or(AttributeResolverError::MissingFunc(func_name))?;

    let (func_binding, created) = FuncBinding::find_or_create(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        args,
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    if created {
        func_binding.execute(txn, nats, veritech).await?;
    }

    Ok((func, func_binding, created))
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
