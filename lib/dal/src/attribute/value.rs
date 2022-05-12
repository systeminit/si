//! An [`AttributeValue`] represents which [`FuncBinding`] and [`FuncBindingReturnValue`] provide
//! attribute's value. Moreover, it tracks whether the value is proxied or not. Proxied values
//! "point" to another [`AttributeValue`] to provide the attribute's value.

use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};

use telemetry::prelude::*;
use thiserror::Error;
use uuid::Uuid;

use crate::func::backend::FuncBackendKind;
use crate::DalContext;
use crate::{
    attribute::context::{AttributeContext, AttributeContextBuilderError, AttributeReadContext},
    attribute::prototype::{AttributePrototype, AttributePrototypeId},
    func::{
        backend::{
            array::FuncBackendArrayArgs, boolean::FuncBackendBooleanArgs,
            integer::FuncBackendIntegerArgs, map::FuncBackendMapArgs,
            prop_object::FuncBackendPropObjectArgs, string::FuncBackendStringArgs,
        },
        binding::{FuncBinding, FuncBindingError, FuncBindingId},
        binding_return_value::{
            FuncBindingReturnValue, FuncBindingReturnValueError, FuncBindingReturnValueId,
        },
    },
    impl_standard_model, pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, standard_model_has_many, Func,
    HistoryEventError, IndexMap, Prop, PropError, PropId, PropKind, ReadTenancyError,
    StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};

const CHILD_ATTRIBUTE_VALUES_IN_CONTEXT: &str =
    include_str!("../queries/attribute_value_child_attribute_values_in_context.sql");
const FIND_FOR_CONTEXT: &str = include_str!("../queries/attribute_value_find_for_context.sql");
const FIND_PROP_FOR_VALUE: &str =
    include_str!("../queries/attribute_value_find_prop_for_value.sql");
const FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_find_with_parent_and_key_for_context.sql");
const FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_find_with_parent_and_prototype_for_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT: &str =
    include_str!("../queries/attribute_value_list_payload_for_read_context.sql");

#[derive(Error, Debug)]
pub enum AttributeValueError {
    #[error("AttributeContextBuilder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("AttributePrototype not found for AttributeValue: {0} ({:?})")]
    AttributePrototypeNotFound(AttributeValueId, Visibility),
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(String),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("FuncBindingReturnValue error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("attribute value not found: {0} ({1:?})")]
    NotFound(AttributeValueId, Visibility),
    #[error("missing attribute value")]
    Missing,
    #[error("func not found: {0}")]
    MissingFunc(String),
    #[error(
        "attribute values must have an associated attribute prototype, and this one does not. bug!"
    )]
    MissingAttributePrototype,
    #[error(
        "parent must be for an array, map, or object prop: attribute resolver id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributeValueId, PropKind),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("Prop not found: {0}")]
    PropNotFound(PropId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
}

pub type AttributeValueResult<T> = Result<T, AttributeValueError>;

pk!(AttributeValuePk);
pk!(AttributeValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    pk: AttributeValuePk,
    id: AttributeValueId,
    func_binding_id: FuncBindingId,
    /// The [`FuncBindingReturnValueId`] that represents the value at this specific position & context.
    func_binding_return_value_id: FuncBindingReturnValueId,
    /// The [`AttributeValueId`] (from a less-specific [`AttributeContext`]) that this
    /// [`AttributeValue`] is standing in for in this more-specific [`AttributeContext`].
    proxy_for_attribute_value_id: Option<AttributeValueId>,
    /// If this is a `sealed_proxy`, then it should **not** update its [`FuncBindingReturnValueId`] from the
    /// [`AttributeValue`] referenced to in `proxy_for_attribute_value_id`.
    sealed_proxy: bool,
    pub index_map: Option<IndexMap>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub context: AttributeContext,
    #[serde(flatten)]
    tenancy: WriteTenancy,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        context: AttributeContext,
        key: Option<String>,
    ) -> AttributeValueResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM attribute_value_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &context,
                    &func_binding_id,
                    &func_binding_return_value_id,
                    &key,
                ],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;

        object.update_parent_index_map(ctx).await?;

        Ok(object)
    }

    standard_model_accessor!(
        proxy_for_attribute_value_id,
        OptionBigInt<AttributeValueId>,
        AttributeValueResult
    );
    standard_model_accessor!(sealed_proxy, bool, AttributeValueResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), AttributeValueResult);
    standard_model_accessor!(
        func_binding_return_value_id,
        Pk(FuncBindingReturnValueId),
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

    standard_model_has_many!(
        lookup_fn: child_attribute_values,
        table: "attribute_value_belongs_to_attribute_value",
        model_table: "attribute_values",
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

    pub async fn update_stored_index_map(
        &self,
        ctx: &DalContext<'_, '_>,
    ) -> AttributeValueResult<()> {
        standard_model::update(
            ctx,
            "attribute_values",
            "index_map",
            self.id(),
            &self.index_map,
            TypeHint::JsonB,
        )
        .await?;
        Ok(())
    }

    /// Returns a list of child [`AttributeValues`](crate::AttributeValue) for a given
    /// [`AttributeValue`] and [`AttributeReadContext`](crate::AttributeReadContext).
    pub async fn child_attribute_values_in_context(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        attribute_read_context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(
                CHILD_ATTRIBUTE_VALUES_IN_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_value_id,
                    &attribute_read_context,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_with_parent_and_prototype_for_context(
        ctx: &DalContext<'_, '_>,
        parent_attribute_value_id: Option<AttributeValueId>,
        attribute_prototype_id: AttributePrototypeId,
        context: AttributeContext,
    ) -> AttributeValueResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &context,
                    &attribute_prototype_id,
                    &parent_attribute_value_id,
                ],
            )
            .await?;

        Ok(standard_model::option_object_from_row(row)?)
    }

    /// Find [`Self`] with a given parent value and key.
    pub async fn find_with_parent_and_key_for_context(
        ctx: &DalContext<'_, '_>,
        parent_attribute_value_id: Option<AttributeValueId>,
        key: Option<String>,
        context: AttributeContext,
    ) -> AttributeValueResult<Option<Self>> {
        let row = ctx
            .pg_txn()
            .query_opt(
                FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &context,
                    &parent_attribute_value_id,
                    &key,
                ],
            )
            .await?;

        Ok(standard_model::option_object_from_row(row)?)
    }

    pub async fn find_for_context(
        ctx: &DalContext<'_, '_>,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[ctx.read_tenancy(), ctx.visibility(), &context],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Return the [`Prop`] that the [`AttributeValueId`] belongs to,
    /// following the relationship through [`AttributePrototype`].
    pub async fn find_prop_for_value(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Prop> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                FIND_PROP_FOR_VALUE,
                &[ctx.read_tenancy(), ctx.visibility(), &attribute_value_id],
            )
            .await?;

        Ok(standard_model::object_from_row(row)?)
    }

    pub async fn list_payload_for_read_context(
        ctx: &DalContext<'_, '_>,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_PAYLOAD_FOR_READ_CONTEXT,
                &[ctx.read_tenancy(), ctx.visibility(), &context],
            )
            .await?;
        let mut result = Vec::new();
        for row in rows.into_iter() {
            let func_binding_return_value_json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: Option<FuncBindingReturnValue> =
                serde_json::from_value(func_binding_return_value_json)?;

            let prop_json: serde_json::Value = row.try_get("prop_object")?;
            let prop: Prop = serde_json::from_value(prop_json)?;

            let attribute_value_json: serde_json::Value = row.try_get("attribute_value_object")?;
            let attribute_value: AttributeValue = serde_json::from_value(attribute_value_json)?;

            let parent_attribute_value_id: Option<AttributeValueId> =
                row.try_get("parent_attribute_value_id")?;

            result.push(AttributeValuePayload::new(
                prop,
                func_binding_return_value,
                attribute_value,
                parent_attribute_value_id,
            ));
        }
        Ok(result)
    }

    /// Update the [`AttributeValue`] for a specific [`AttributeContext`] to the given value. If the
    /// given [`AttributeValue`] is for a different [`AttributeContext`] than the one provided, a
    /// new [`AttributeValue`] will be created for the given [`AttributeContext`].
    ///
    /// By passing in [`None`] as the `value`, the caller is explicitly saying "this value does not
    /// exist here". This is potentially useful for "tombstoning" values that have been inherited
    /// from a less-specific [`AttributeContext`]. For example, if a value has been set for a
    /// [`SchemaVariant`](crate::SchemaVariant), but we do not want that value to exist for a
    /// specific [`Component`](crate::Component), we can update the variant's value to [`None`] in
    /// an [`AttributeContext`] specific to that component.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_for_context(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        context: AttributeContext,
        value: Option<serde_json::Value>,
        // TODO: Allow updating the key
        _key: Option<String>,
    ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
        let given_attribute_value = Self::get_by_id(ctx, &attribute_value_id)
            .await?
            .ok_or_else(|| AttributeValueError::NotFound(attribute_value_id, *ctx.visibility()))?;

        let original_attribute_prototype = given_attribute_value
            .attribute_prototype_with_tenancy(ctx)
            .await?
            .ok_or_else(|| {
                AttributeValueError::AttributePrototypeNotFound(
                    attribute_value_id,
                    *ctx.visibility(),
                )
            })?;

        // If the AttributeValue we were given isn't for the _specific_ context that we're trying to
        // update, make a new one. This is necessary, since the one that we were given might be the
        // "default" one that is directly attached to a Prop, or the one from a SchemaVariant, and the
        // AttributeContext might be requesting that we set the value in a more specific context.
        let mut attribute_value = if given_attribute_value.context == context {
            given_attribute_value
        } else {
            let av = Self::new(
                ctx,
                given_attribute_value.func_binding_id(),
                given_attribute_value.func_binding_return_value_id(),
                context,
                given_attribute_value.key,
            )
            .await?;

            if let Some(parent_attribute_value_id) = parent_attribute_value_id {
                av.set_parent_attribute_value(ctx, &parent_attribute_value_id)
                    .await?;
            }

            av
        };

        let prop = AttributeValue::find_prop_for_value(ctx, *attribute_value.id()).await?;

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

        let (func, func_binding, created) = set_value(ctx, func_name, func_args).await?;
        let func_binding_return_value = if created {
            func_binding.execute(ctx).await?
        } else {
            FuncBindingReturnValue::get_by_func_binding_id_or_execute(ctx, *func_binding.id())
                .await?
        };

        attribute_value
            .set_func_binding_id(ctx, *func_binding.id())
            .await?;

        let attribute_prototype_id = AttributePrototype::update_for_context(
            ctx,
            *original_attribute_prototype.id(),
            context,
            *func.id(),
            *func_binding.id(),
            *func_binding_return_value.id(),
            parent_attribute_value_id,
            None,
            Some(*attribute_value.id()),
        )
        .await
        .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;
        attribute_value.unset_attribute_prototype(ctx).await?;
        attribute_value
            .set_attribute_prototype(ctx, &attribute_prototype_id)
            .await?;

        attribute_value
            .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
            .await?;

        attribute_value.update_parent_index_map(ctx).await?;

        Ok((value, *attribute_value.id()))
    }

    /// Insert a new value under the parent [`AttributeValue`] in the given [`AttributeContext`]. This is mostly only
    /// useful for adding elements to a [`PropKind::Array`], or to a [`PropKind::Map`]. Updating existing values in an
    /// [`Array`](PropKind::Array), or [`Map`](PropKind::Map), and setting/updating all other [`PropKind`] should be
    /// able to directly use [`update_for_context()`](AttributeValue::update_for_context()), as there will already be an
    /// appropriate [`AttributeValue`] to use. By using this function,
    /// [`update_for_context()`](AttributeValue::update_for_context()) is called after we have created an appropriate
    /// [`AttributeValue`] to use.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn insert_for_context(
        ctx: &DalContext<'_, '_>,
        context: AttributeContext,
        parent_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        let parent_prop =
            AttributeValue::find_prop_for_value(ctx, parent_attribute_value_id).await?;

        let key = if let Some(k) = key {
            Some(k)
        } else if *parent_prop.kind() == PropKind::Array {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };

        let unset_func_name = "si:unset".to_string();
        let unset_func = Func::find_by_attr(ctx, "name", &unset_func_name)
            .await?
            .pop()
            .ok_or(AttributeValueError::MissingFunc(unset_func_name))?;
        let (unset_func_binding, created) = FuncBinding::find_or_create(
            ctx,
            serde_json::json![null],
            *unset_func.id(),
            FuncBackendKind::Unset,
        )
        .await?;
        let func_binding_return_value = if created {
            unset_func_binding.execute(ctx).await?
        } else {
            FuncBindingReturnValue::get_by_func_binding_id_or_execute(ctx, *unset_func_binding.id())
                .await?
        };

        let attribute_value = Self::new(
            ctx,
            *unset_func_binding.id(),
            *func_binding_return_value.id(),
            context,
            key.clone(),
        )
        .await?;

        if AttributePrototype::find_with_parent_value_and_key_for_context(
            ctx,
            Some(parent_attribute_value_id),
            key.clone(),
            context,
        )
        .await
        .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?
        .is_none()
        {
            AttributePrototype::new_with_existing_value(
                ctx,
                *unset_func.id(),
                context,
                key.clone(),
                Some(parent_attribute_value_id),
                None,
                *attribute_value.id(),
            )
            .await
            .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;
        };

        let (_, attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(parent_attribute_value_id),
            context,
            value,
            key,
        )
        .await?;

        Ok(attribute_value_id)
    }

    #[instrument(skip_all)]
    pub async fn update_parent_index_map(
        &self,
        ctx: &DalContext<'_, '_>,
    ) -> AttributeValueResult<()> {
        if let Some(mut parent_value) = self.parent_attribute_value(ctx).await? {
            let parent_prop = Prop::get_by_id(ctx, &parent_value.context.prop_id())
                .await?
                .ok_or_else(|| AttributeValueError::PropNotFound(parent_value.context.prop_id()))?;

            if *parent_prop.kind() == PropKind::Array || *parent_prop.kind() == PropKind::Map {
                match parent_value.index_map_mut() {
                    Some(index_map) => {
                        index_map.push(*self.id(), self.key.clone());
                    }
                    None => {
                        let mut index_map = IndexMap::new();
                        index_map.push(*self.id(), self.key.clone());
                        parent_value.index_map = Some(index_map);
                    }
                }
                parent_value.update_stored_index_map(ctx).await?;
            }
        };

        Ok(())
    }

    /// Ensure the [`AttributeValueId`] has a "set" value in the given [`AttributeContext`], doing the same for its
    /// parent [`AttributeValue`], and return the [`AttributeValueId`] for [`Self`] (which may be different from what
    /// was passed in.
    #[instrument(skip_all)]
    #[async_recursion]
    async fn vivify_value_and_parent_values(
        ctx: &DalContext<'_, '_>,
        context: AttributeContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueId> {
        let attribute_value = Self::get_by_id(ctx, &attribute_value_id)
            .await?
            .ok_or(AttributeValueError::Missing)?;

        // If we're already set, there's not anything for us to do.
        if FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id)
            .await?
            .ok_or_else(|| {
                AttributeValueError::UnableToCreateParent(format!(
                    "Missing FuncBindingReturnValue for AttributeValue: {}",
                    attribute_value_id
                ))
            })?
            .value()
            .is_some()
        {
            return Ok(attribute_value_id);
        }

        let maybe_parent_attribute_value_id = attribute_value
            .parent_attribute_value(ctx)
            .await?
            .map(|av| *av.id());

        let (_, new_attribute_value_id) = Self::update_for_context(
            ctx,
            attribute_value_id,
            maybe_parent_attribute_value_id,
            context,
            Some(json![{}]),
            None,
        )
        .await?;

        Ok(new_attribute_value_id)
    }

    #[async_recursion]
    async fn populate_child_proxies_for_value(
        ctx: &DalContext<'_, '_>,
        original_attribute_value_id: AttributeValueId,
        previous_write_context: AttributeContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let read_context = AttributeReadContext {
            prop_id: None,
            ..AttributeReadContext::from(previous_write_context)
        };
        // These are the values that we wish to create proxies for in our new context.
        let original_child_values =
            Self::child_attribute_values_in_context(ctx, original_attribute_value_id, read_context)
                .await?;

        for original_child_value in original_child_values {
            let mut write_context_builder = AttributeContextBuilder::from(previous_write_context);
            let write_context = write_context_builder
                .set_prop_id(original_child_value.context.prop_id())
                .to_context()?;

            if original_child_value.context == write_context {
                // The `AttributeValue` that we found is one that was already
                // set in the desired `AttributeContext`, but its parent was
                // from a less-specific `AttributeContext`. Since it now has
                // an appropriate parent `AttributeValue` within the desired
                // `AttributeContext`, we need to have it under that parent
                // instead of the old one.
                original_child_value
                    .unset_parent_attribute_value(ctx)
                    .await?;
                original_child_value
                    .set_parent_attribute_value(ctx, &attribute_value_id)
                    .await?;
            } else {
                // Since there isn't already an `AttributeValue` to represent
                // the one from a less-specific `AttributeContext`, we need
                // to create a proxy `AttributeValue` in the desired
                // `AttributeContext` so that we can do things like add it to
                // the `IndexMap` of the parent (that exists in the desired
                // context).
                let mut child_value = Self::new(
                    ctx,
                    original_child_value.func_binding_id(),
                    original_child_value.func_binding_return_value_id(),
                    write_context,
                    original_child_value.key(),
                )
                .await?;
                let child_attribute_value_prototype = original_child_value
                    .attribute_prototype(ctx)
                    .await?
                    .ok_or_else(|| {
                        AttributeValueError::AttributePrototypeNotFound(
                            *original_child_value.id(),
                            *ctx.visibility(),
                        )
                    })?;
                child_value
                    .set_attribute_prototype(ctx, child_attribute_value_prototype.id())
                    .await?;
                child_value
                    .set_parent_attribute_value(ctx, &attribute_value_id)
                    .await?;
                child_value
                    .set_proxy_for_attribute_value_id(ctx, Some(*original_child_value.id()))
                    .await?;

                // Now that we've created a proxy `AttributeValue`, we need
                // to create proxies for all of the original value's children.
                Self::populate_child_proxies_for_value(
                    ctx,
                    *original_child_value.id(),
                    write_context,
                    *child_value.id(),
                )
                .await?;
            }
        }

        Ok(())
    }

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

fn as_type<T: serde::de::DeserializeOwned>(json: serde_json::Value) -> AttributeValueResult<T> {
    T::deserialize(&json).map_err(|_| {
        AttributeValueError::InvalidPropValue(std::any::type_name::<T>().to_owned(), json)
    })
}

async fn set_value(
    ctx: &DalContext<'_, '_>,
    func_name: &str,
    args: serde_json::Value,
) -> AttributeValueResult<(Func, FuncBinding, bool)> {
    let func_name = func_name.to_owned();
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or(AttributeValueError::MissingFunc(func_name))?;

    let (func_binding, created) =
        FuncBinding::find_or_create(ctx, args, *func.id(), *func.backend_kind()).await?;

    Ok((func, func_binding, created))
}

#[derive(Debug)]
pub struct AttributeValuePayload {
    pub prop: Prop,
    pub func_binding_return_value: Option<FuncBindingReturnValue>,
    pub attribute_value: AttributeValue,
    pub parent_attribute_value_id: Option<AttributeValueId>,
}

impl AttributeValuePayload {
    pub fn new(
        prop: Prop,
        func_binding_return_value: Option<FuncBindingReturnValue>,
        attribute_value: AttributeValue,
        parent_attribute_value_id: Option<AttributeValueId>,
    ) -> Self {
        Self {
            prop,
            func_binding_return_value,
            attribute_value,
            parent_attribute_value_id,
        }
    }
}
