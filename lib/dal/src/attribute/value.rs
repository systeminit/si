//! An [`AttributeValue`] represents which [`FuncBinding`] and [`FuncBindingReturnValue`] provide
//! attribute's value. Moreover, it tracks whether the value is proxied or not. Proxied values
//! "point" to another [`AttributeValue`] to provide the attribute's value.

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_data::{NatsError, PgError};
use std::collections::{HashMap, HashSet, VecDeque};
use telemetry::prelude::*;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    attribute::{
        context::{
            AttributeContext, AttributeContextBuilder, AttributeContextBuilderError,
            AttributeReadContext,
        },
        prototype::{AttributePrototype, AttributePrototypeId},
        value::dependent_update::collection::AttributeValueDependentCollectionHarness,
    },
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
    JobError,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    AttributeContextError, AttributePrototypeArgumentError, CodeGenerationJob, ComponentId,
    DalContext, Func, FuncError, HistoryEventError, IndexMap, InternalProviderId, Prop, PropError,
    PropId, PropKind, ReadTenancyError, StandardModel, StandardModelError, Timestamp,
    TransactionsError, UpdateDependentValuesJob, Visibility, WriteTenancy,
};

pub mod view;

// For finding dependent_update attribute values based on providers.
pub mod dependent_update;

const CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_child_attribute_values_for_context.sql");
const CHILD_ATTRIBUTE_VALUES_FOR_EXACT_CONTEXT: &str =
    include_str!("../queries/attribute_value_child_attribute_values_for_exact_context.sql");
const LIST_FOR_CONTEXT: &str = include_str!("../queries/attribute_value_list_for_context.sql");
const FIND_PROP_FOR_VALUE: &str =
    include_str!("../queries/attribute_value_find_prop_for_value.sql");
const FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_find_with_parent_and_key_for_context.sql");
const FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_find_with_parent_and_prototype_for_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT: &str =
    include_str!("../queries/attribute_value_list_payload_for_read_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT: &str =
    include_str!("../queries/attribute_value_list_payload_for_read_context_and_root.sql");

#[derive(Error, Debug)]
pub enum AttributeValueError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
    #[error("AttributeContext error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("AttributeContextBuilder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(String),
    #[error("AttributePrototypeArgument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("AttributePrototype not found for AttributeValue: {0} ({1:?})")]
    AttributePrototypeNotFound(AttributeValueId, Visibility),
    #[error("invalid json pointer: {0} for {1}")]
    BadJsonPointer(String, String),
    #[error("job error: {0}")]
    Job(#[from] JobError),
    #[error("component error: {0}")]
    Component(String),
    #[error("component not found for id: {0}")]
    ComponentNotFound(ComponentId),
    #[error("empty attribute prototype arguments for group name: {0}")]
    EmptyAttributePrototypeArgumentsForGroup(String),
    #[error("external provider error: {0}")]
    ExternalProvider(String),
    #[error("found duplicate attribute value ({0}) for self ({1}) for parent: {2}")]
    FoundDuplicateForParent(AttributeValueId, AttributeValueId, AttributeValueId),
    #[error("found duplicate attribute value ({0}) when creating new attribute value in provider context: {1:?}")]
    FoundDuplicateForProviderContext(AttributeValueId, AttributeContext),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("FuncBindingReturnValue error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("FuncBindingReturnValue not found for AttributeValue: {0}")]
    FuncBindingReturnValueNotFound(AttributeValueId, Visibility),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("{0}")]
    IncompatibleAttributeReadContext(&'static str),
    #[error("internal provider error: {0}")]
    InternalProvider(String),
    #[error("internal provider not found by id: {0}")]
    InternalProviderNotFound(InternalProviderId),
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
    #[error("found invalid object value fields not found in corresponding prop: {0:?}")]
    InvalidObjectValueFields(Vec<String>),
    #[error("json pointer missing for attribute view {0:?} {1:?}")]
    JsonPointerMissing(AttributeValueId, HashMap<AttributeValueId, String>),
    #[error("missing attribute value")]
    Missing,
    #[error(
        "attribute values must have an associated attribute prototype, and this one does not. bug!"
    )]
    MissingAttributePrototype,
    #[error("expected prop id {0} to have a child")]
    MissingChildProp(PropId),
    #[error("missing attribute value with id: {0}")]
    MissingForId(AttributeValueId),
    #[error("func not found: {0}")]
    MissingFunc(String),
    #[error("FuncBinding not found: {0}")]
    MissingFuncBinding(FuncBindingId),
    #[error("func binding return value not found")]
    MissingFuncBindingReturnValue,
    #[error("missing value from func binding return value for attribute value id: {0}")]
    MissingValueFromFuncBindingReturnValue(AttributeValueId),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("attribute value not found: {0} ({1:?})")]
    NotFound(AttributeValueId, Visibility),
    #[error("missing attribute value for external provider context: {0:?}")]
    NotFoundForExternalProviderContext(AttributeContext),
    #[error("missing attribute value for internal provider context: {0:?}")]
    NotFoundForInternalProviderContext(AttributeContext),
    #[error("using json pointer for attribute view yielded no value")]
    NoValueForJsonPointer,
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
    #[error("schema not found for component id: {0}")]
    SchemaNotFoundForComponent(ComponentId),
    #[error("schema variant not found for component id: {0}")]
    SchemaVariantNotFoundForComponent(ComponentId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("Unable to create parent AttributeValue: {0}")]
    UnableToCreateParent(String),
    #[error("the root prop id stack cannot be empty while work queue is not empty")]
    UnexpectedEmptyRootStack,
    #[error("unexpected prop kind: {0}")]
    UnexpectedPropKind(PropKind),
    #[error("JSON value failed to parse as an array")]
    ValueAsArray,
    #[error("JSON value failed to parse as an map")]
    ValueAsMap,
    #[error("JSON value failed to parse as an object")]
    ValueAsObject,
    #[error("schema variant missing in context")]
    SchemaVariantMissing,
    #[error("schema missing in context")]
    SchemaMissing,
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
        key: Option<impl Into<String>>,
    ) -> AttributeValueResult<Self> {
        // If we are trying to create values in a provider context, they will not have a parent,
        // so we will need to ensure they do not exist in the same context.
        if context.is_least_specific_field_kind_internal_or_external_provider()? {
            if let Some(found_attribute_value) =
                AttributeValue::find_for_context(ctx, context.into()).await?
            {
                if found_attribute_value.context == context {
                    return Err(AttributeValueError::FoundDuplicateForProviderContext(
                        found_attribute_value.id,
                        context,
                    ));
                }
            }
        }

        let key: Option<String> = key.map(|s| s.into());
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
        set_fn: set_parent_attribute_value_unchecked,
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

    pub async fn set_parent_attribute_value(
        &self,
        ctx: &DalContext<'_, '_>,
        belongs_to_id: &AttributeValueId,
    ) -> AttributeValueResult<()> {
        if let Some(potential_duplicate) = Self::find_with_parent_and_key_for_context(
            ctx,
            Some(*belongs_to_id),
            self.key.clone(),
            self.context.into(),
        )
        .await?
        {
            if potential_duplicate.context == self.context {
                return Err(AttributeValueError::FoundDuplicateForParent(
                    potential_duplicate.id,
                    self.id,
                    *belongs_to_id,
                ));
            }
        }
        self.set_parent_attribute_value_unchecked(ctx, belongs_to_id)
            .await?;
        Ok(())
    }

    pub fn index_map_mut(&mut self) -> Option<&mut IndexMap> {
        self.index_map.as_mut()
    }

    /// Returns the [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
    /// corresponding to the field on [`Self`].
    pub async fn get_value(
        &self,
        ctx: &DalContext<'_, '_>,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
            Some(func_binding_return_value) => Ok(func_binding_return_value.value().cloned()),
            None => Err(AttributeValueError::MissingFuncBindingReturnValue),
        }
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
    pub async fn child_attribute_values_for_context(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        attribute_read_context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(
                CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT,
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

    async fn child_attribute_values_for_exact_context(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        attribute_read_context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(
                CHILD_ATTRIBUTE_VALUES_FOR_EXACT_CONTEXT,
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
        context: AttributeReadContext,
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

    /// List [`AttributeValues`](crate::AttributeValue) for a provided
    /// [`AttributeReadContext`](crate::AttributeReadContext).
    ///
    /// If you only anticipate one result to be returned and have an
    /// [`AttributeReadContext`](crate::AttributeReadContext)
    /// that is also a valid [`AttributeContext`](crate::AttributeContext), then you should use
    /// [`Self::find_for_context()`] instead of this method.
    ///
    /// This does _not_ work for maps and arrays, barring the _first_ instance of the array or map
    /// object themselves! For those objects, please use
    /// [`Self::find_with_parent_and_key_for_context()`].
    pub async fn list_for_context(
        ctx: &DalContext<'_, '_>,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[ctx.read_tenancy(), ctx.visibility(), &context],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find one [`AttributeValue`](crate::AttributeValue) for a provided
    /// [`AttributeReadContext`](crate::AttributeReadContext).
    ///
    /// This is a modified version of [`Self::list_for_context()`] that requires an
    /// [`AttributeReadContext`](crate::AttributeReadContext)
    /// that is also a valid [`AttributeContext`](crate::AttributeContext) _and_ "pops" the first
    /// row off the rows found (which are sorted from most to least specific). Thus, the "popped"
    /// row will corresponding to the most specific [`AttributeValue`] found.
    ///
    /// This does _not_ work for maps and arrays, barring the _first_ instance of the array or map
    /// object themselves! For those objects, please use
    /// [`Self::find_with_parent_and_key_for_context()`].
    pub async fn find_for_context(
        ctx: &DalContext<'_, '_>,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Option<Self>> {
        AttributeContextBuilder::from(context).to_context()?;
        let mut rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[ctx.read_tenancy(), ctx.visibility(), &context],
            )
            .await?;
        let maybe_row = rows.pop();
        Ok(standard_model::option_object_from_row(maybe_row)?)
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
        let schema_variant_id = if let Some(schema_variant_id) = context.schema_variant_id() {
            schema_variant_id
        } else {
            return Err(AttributeValueError::SchemaVariantMissing);
        };
        if !context.has_schema_id() {
            return Err(AttributeValueError::SchemaMissing);
        }

        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_PAYLOAD_FOR_READ_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &context,
                    &schema_variant_id,
                ],
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

    /// This method is similar to [`Self::list_payload_for_read_context()`], but it leverages a
    /// root [`AttributeValueId`](crate::AttributeValue) in order to find payloads at any
    /// root [`Prop`](crate::Prop) corresponding to the provided context and root value.
    ///
    /// Requirements for the [`AttributeReadContext`](crate::AttributeReadContext):
    /// - [`PropId`](crate::Prop) must be set to [`None`]
    /// - Both providers fields must be unset
    pub async fn list_payload_for_read_context_and_root(
        ctx: &DalContext<'_, '_>,
        root_attribute_value_id: AttributeValueId,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
        if context.has_prop_id()
            || !context.has_unset_internal_provider()
            || !context.has_unset_external_provider()
        {
            return Err(AttributeValueError::IncompatibleAttributeReadContext("incompatible attribute read context for query: prop must be empty and providers must be unset"));
        }

        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &context,
                    &root_attribute_value_id,
                ],
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
    ///
    /// This method returns the following:
    /// - the [`Option<serde_json::Value>`] that was passed in
    /// - the updated [`AttributeValueId`](Self)
    pub async fn update_for_context(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        context: AttributeContext,
        value: Option<serde_json::Value>,
        // TODO: Allow updating the key
        key: Option<String>,
    ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
        Self::update_for_context_raw(
            ctx,
            attribute_value_id,
            parent_attribute_value_id,
            context,
            value,
            key,
            true,
        )
        .await
    }

    pub async fn update_for_context_without_creating_proxies(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        context: AttributeContext,
        value: Option<serde_json::Value>,
        // TODO: Allow updating the key
        key: Option<String>,
    ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
        Self::update_for_context_raw(
            ctx,
            attribute_value_id,
            parent_attribute_value_id,
            context,
            value,
            key,
            false,
        )
        .await
    }

    async fn update_for_context_raw(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        context: AttributeContext,
        value: Option<serde_json::Value>,
        // TODO: Allow updating the key
        _key: Option<String>,
        create_child_proxies: bool,
    ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
        let mut maybe_parent_attribute_value_id = parent_attribute_value_id;

        let given_attribute_value = Self::get_by_id(ctx, &attribute_value_id)
            .await?
            .ok_or_else(|| AttributeValueError::NotFound(attribute_value_id, *ctx.visibility()))?;

        let original_attribute_prototype = given_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or_else(|| {
                AttributeValueError::AttributePrototypeNotFound(
                    attribute_value_id,
                    *ctx.visibility(),
                )
            })?;

        // We need to make sure that all of the parents "exist" (are not the "unset" value).
        // We can't rely on the client having created/set all of the parents already, as the
        // parent might be an Object, or an Array/Map (instead of an element in an Array/Map).
        // The client will only be creating new elements in Arrays/Maps, and not
        // Objects/Arrays/Maps themselves.
        if let Some(parent_attribute_value_id) = maybe_parent_attribute_value_id {
            let parent_attribute_value = Self::get_by_id(ctx, &parent_attribute_value_id)
                .await?
                .ok_or_else(|| {
                    AttributeValueError::NotFound(parent_attribute_value_id, *ctx.visibility())
                })?;
            let mut parent_attribute_context_builder = AttributeContextBuilder::from(context);
            let parent_attribute_context = parent_attribute_context_builder
                .set_prop_id(parent_attribute_value.context.prop_id())
                .to_context()?;
            let maybe = Self::vivify_value_and_parent_values_raw(
                ctx,
                parent_attribute_context,
                parent_attribute_value_id,
                create_child_proxies,
            )
            .await?;
            maybe_parent_attribute_value_id = Some(maybe);
        }

        // If the AttributeValue we were given isn't for the _specific_ context that we're trying to
        // update, make a new one. This is necessary, since the one that we were given might be the
        // "default" one that is directly attached to a Prop, or the one from a SchemaVariant, and the
        // AttributeContext might be requesting that we set the value in a more specific context.
        let mut attribute_value = if given_attribute_value.context == context {
            given_attribute_value
        } else {
            // Check if we created an appropriate AttributeValue in the process of vivifying
            // the parent `AttributeValue`s, and populating proxy `AttributeValue`s for their
            // child `AttributeValue`s.
            let maybe_value = match Self::find_with_parent_and_key_for_context(
                ctx,
                maybe_parent_attribute_value_id,
                given_attribute_value.key.clone(),
                context.into(),
            )
            .await?
            {
                // If the value we found is of the `AttributeContext` that we want to
                // update, then use it.
                Some(value) if value.context == context => Some(value),
                // Anything else isn't appropriate to use, so pretend that we didn't
                // find anything, whether or not we did.
                _ => None,
            };

            let av = if let Some(value) = maybe_value {
                value
            } else {
                // We haven't found an appropriate `AttributeValue` to use, so we
                // need to make one.
                let value = Self::new(
                    ctx,
                    given_attribute_value.func_binding_id(),
                    given_attribute_value.func_binding_return_value_id(),
                    context,
                    given_attribute_value.key.clone(),
                )
                .await?;
                if let Some(parent_attribute_value_id) = maybe_parent_attribute_value_id {
                    value
                        .set_parent_attribute_value(ctx, &parent_attribute_value_id)
                        .await?;
                }

                // Whenever we make a new `AttributeValue` we need to create
                // proxies to represent the children of the parallel `AttributeValue`
                // that exists in a different `AttributeContext`.
                if create_child_proxies {
                    Self::populate_child_proxies_for_value(
                        ctx,
                        *given_attribute_value.id(),
                        context,
                        *value.id(),
                    )
                    .await?;
                }

                value
            };

            av
        };

        let prop = AttributeValue::find_prop_for_value(ctx, *attribute_value.id()).await?;

        let (func_name, func_args) = match (prop.kind(), value.clone()) {
            (_, None) => ("si:unset", serde_json::to_value(())?),
            (PropKind::Array, Some(value_json)) => {
                let value: Vec<serde_json::Value> = as_type(value_json)?;
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
            (PropKind::Map, Some(value_json)) => {
                let value: serde_json::Map<String, serde_json::Value> = as_type(value_json)?;
                (
                    "si:setMap",
                    serde_json::to_value(FuncBackendMapArgs::new(value))?,
                )
            }
            (PropKind::Object, Some(value_json)) => {
                let value: serde_json::Map<String, serde_json::Value> = as_type(value_json)?;
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

        let (func, func_binding, func_binding_return_value) =
            set_value(ctx, func_name, func_args).await?;

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
            maybe_parent_attribute_value_id,
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

        // If the value we just updated is a proxy, we need to seal it to prevent
        // it from being automatically updated from the `AttributeValue` it is
        // proxying, since we're overrode that value.
        if attribute_value.proxy_for_attribute_value_id().is_some() {
            attribute_value.set_sealed_proxy(ctx, true).await?;
        }

        attribute_value.update_parent_index_map(ctx).await?;

        // Do we need to process the unprocessed value and populate nested values?
        // If the unprocessed value doesn't equal the value then we have a populated "container"
        // (i.e. object, map, array) that contains values which need to be made into
        // AttributeValues of their own.
        if func_binding_return_value.unprocessed_value() != func_binding_return_value.value() {
            if let Some(unprocessed_value) = func_binding_return_value.unprocessed_value().cloned()
            {
                Self::populate_nested_values(
                    ctx,
                    *attribute_value.id(),
                    context,
                    unprocessed_value,
                )
                .await?;
            }
        }

        if context.component_id().is_some() {
            ctx.enqueue_job(CodeGenerationJob::new(
                context.component_id(),
                context.system_id(),
            ))
            .await?;
        }

        let dependent_attribute_values =
            AttributeValueDependentCollectionHarness::collect(ctx, attribute_value.context).await?;
        for dependent_attribute_value in dependent_attribute_values {
            ctx.enqueue_job(UpdateDependentValuesJob::new(
                *dependent_attribute_value.id(),
            ))
            .await?;
        }

        Ok((value, *attribute_value.id()))
    }

    /// Insert a new value under the parent [`AttributeValue`] in the given [`AttributeContext`]. This is mostly only
    /// useful for adding elements to a [`PropKind::Array`], or to a [`PropKind::Map`]. Updating existing values in an
    /// [`Array`](PropKind::Array), or [`Map`](PropKind::Map), and setting/updating all other [`PropKind`] should be
    /// able to directly use [`update_for_context()`](AttributeValue::update_for_context()), as there will already be an
    /// appropriate [`AttributeValue`] to use. By using this function,
    /// [`update_for_context()`](AttributeValue::update_for_context()) is called after we have created an appropriate
    /// [`AttributeValue`] to use.
    #[instrument(skip_all)]
    pub async fn insert_for_context(
        ctx: &DalContext<'_, '_>,
        parent_context: AttributeContext,
        parent_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        Self::insert_for_context_raw(
            ctx,
            parent_context,
            parent_attribute_value_id,
            value,
            key,
            true,
        )
        .await
    }

    #[instrument(skip_all)]
    pub async fn insert_for_context_without_creating_proxies(
        ctx: &DalContext<'_, '_>,
        parent_context: AttributeContext,
        parent_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        Self::insert_for_context_raw(
            ctx,
            parent_context,
            parent_attribute_value_id,
            value,
            key,
            false,
        )
        .await
    }

    #[instrument(skip_all)]
    async fn insert_for_context_raw(
        ctx: &DalContext<'_, '_>,
        parent_context: AttributeContext,
        parent_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
        create_child_proxies: bool,
    ) -> AttributeValueResult<AttributeValueId> {
        let parent_prop =
            AttributeValue::find_prop_for_value(ctx, parent_attribute_value_id).await?;
        // We can only "insert" new values into Arrays & Maps. All other `PropKind` should be updated directly.
        if *parent_prop.kind() != PropKind::Array && *parent_prop.kind() != PropKind::Map {
            return Err(AttributeValueError::UnexpectedPropKind(*parent_prop.kind()));
        }

        let child_prop = parent_prop
            .child_props(ctx)
            .await?
            .pop()
            .ok_or_else(|| AttributeValueError::MissingChildProp(*parent_prop.id()))?;
        let mut child_context_builder = AttributeContextBuilder::from(parent_context);
        let context = child_context_builder
            .set_prop_id(*child_prop.id())
            .to_context()?;

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
        let (unset_func_binding, unset_func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(ctx, serde_json::json![null], *unset_func.id())
                .await?;

        let attribute_value = Self::new(
            ctx,
            *unset_func_binding.id(),
            *unset_func_binding_return_value.id(),
            context,
            key.clone(),
        )
        .await?;

        AttributePrototype::new_with_existing_value(
            ctx,
            *unset_func.id(),
            context,
            key.clone(),
            Some(parent_attribute_value_id),
            *attribute_value.id(),
        )
        .await
        .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;

        // Create unset AttributePrototypes & AttributeValues for child Props
        // up until (inclusive) we reach an Array/Map.
        let prop = Self::find_prop_for_value(ctx, *attribute_value.id()).await?;
        if *prop.kind() == PropKind::Object {
            let mut child_props: VecDeque<_> = prop
                .child_props(ctx)
                .await?
                .into_iter()
                .map(|p| (*attribute_value.id(), p))
                .collect();
            while let Some((parent_attribute_value_id, prop)) = child_props.pop_front() {
                let context = child_context_builder.set_prop_id(*prop.id()).to_context()?;
                let prop_attribute_value = Self::new(
                    ctx,
                    *unset_func_binding.id(),
                    *unset_func_binding_return_value.id(),
                    context,
                    Option::<&str>::None,
                )
                .await?;

                prop_attribute_value
                    .set_parent_attribute_value(ctx, &parent_attribute_value_id)
                    .await?;

                let prototype = AttributePrototype::new_with_existing_value(
                    ctx,
                    *unset_func.id(),
                    context,
                    None,
                    Some(parent_attribute_value_id),
                    parent_attribute_value_id,
                )
                .await
                .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;

                prop_attribute_value
                    .set_attribute_prototype(ctx, prototype.id())
                    .await?;

                // PropKind::Object is the only kind of Prop that can have child Props,
                // and that we want to create unset AttributePrototypes & AttributeValue
                // for its children.
                if *prop.kind() == PropKind::Object {
                    child_props.extend(
                        prop.child_props(ctx)
                            .await?
                            .into_iter()
                            .map(|p| (*prop_attribute_value.id(), p)),
                    );
                }
            }
        }

        let (_, attribute_value_id) = Self::update_for_context_raw(
            ctx,
            *attribute_value.id(),
            Some(parent_attribute_value_id),
            context,
            value,
            key,
            create_child_proxies,
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
        Self::vivify_value_and_parent_values_raw(ctx, context, attribute_value_id, true).await
    }

    #[instrument(skip_all)]
    #[async_recursion]
    async fn vivify_value_and_parent_values_without_child_proxies(
        ctx: &DalContext<'_, '_>,
        context: AttributeContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueId> {
        Self::vivify_value_and_parent_values_raw(ctx, context, attribute_value_id, false).await
    }

    #[instrument(skip_all)]
    #[async_recursion]
    async fn vivify_value_and_parent_values_raw(
        ctx: &DalContext<'_, '_>,
        context: AttributeContext,
        attribute_value_id: AttributeValueId,
        create_child_proxies: bool,
    ) -> AttributeValueResult<AttributeValueId> {
        let attribute_value = Self::get_by_id(ctx, &attribute_value_id)
            .await?
            .ok_or(AttributeValueError::Missing)?;
        let prop = Prop::get_by_id(ctx, &attribute_value.context.prop_id())
            .await?
            .ok_or_else(|| AttributeValueError::PropNotFound(attribute_value.context.prop_id()))?;
        let empty_value = match prop.kind() {
            PropKind::Array => json!([]),
            PropKind::Object | PropKind::Map => json!({}),
            PropKind::String | PropKind::Boolean | PropKind::Integer => todo!(),
        };

        let unset_func_name = "si:unset".to_string();
        let unset_func = Func::find_by_attr(ctx, "name", &unset_func_name)
            .await?
            .pop()
            .ok_or(AttributeValueError::MissingFunc(unset_func_name))?;

        let func_binding = FuncBinding::get_by_id(ctx, &attribute_value.func_binding_id())
            .await?
            .ok_or(AttributeValueError::MissingFuncBinding(
                attribute_value.func_binding_id,
            ))?;
        let func = func_binding.func(ctx).await?.ok_or_else(|| {
            AttributeValueError::MissingFunc(format!(
                "Func for FuncBindingId {} not found",
                func_binding.id()
            ))
        })?;

        // If we're already set, there might not be anything for us to do.
        if func.id() != unset_func.id() {
            if *prop.kind() == PropKind::Array || *prop.kind() == PropKind::Map {
                // If the Prop is an Array or a Map, we need it to be set in the specific
                // context we're looking at.
                if attribute_value.context == context {
                    return Ok(attribute_value_id);
                }
            } else {
                return Ok(attribute_value_id);
            }
        }

        let maybe_parent_attribute_value_id = attribute_value
            .parent_attribute_value(ctx)
            .await?
            .map(|av| *av.id());

        let (_, new_attribute_value_id) = Self::update_for_context_raw(
            ctx,
            attribute_value_id,
            maybe_parent_attribute_value_id,
            context,
            Some(empty_value),
            None,
            create_child_proxies,
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
        let original_child_values = Self::child_attribute_values_for_context(
            ctx,
            original_attribute_value_id,
            read_context,
        )
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

    #[async_recursion]
    async fn populate_nested_values(
        ctx: &DalContext<'_, '_>,
        parent_attribute_value_id: AttributeValueId,
        update_context: AttributeContext,
        mut unprocessed_value: serde_json::Value,
    ) -> AttributeValueResult<()> {
        let parent_attribute_value = Self::get_by_id(ctx, &parent_attribute_value_id)
            .await?
            .ok_or(AttributeValueError::Missing)?;
        let parent_prop = Prop::get_by_id(ctx, &parent_attribute_value.context.prop_id())
            .await?
            .ok_or_else(|| {
                AttributeValueError::PropNotFound(parent_attribute_value.context.prop_id())
            })?;

        let child_values_for_exact_context = Self::child_attribute_values_for_exact_context(
            ctx,
            *parent_attribute_value.id(),
            AttributeReadContext {
                prop_id: None,
                ..parent_attribute_value.context.into()
            },
        )
        .await?;
        for child_value in child_values_for_exact_context {
            Self::remove_value_and_children(ctx, child_value).await?;
        }

        match parent_prop.kind() {
            PropKind::Object => {
                let unprocessed_object = unprocessed_value
                    .as_object_mut()
                    .ok_or(AttributeValueError::ValueAsObject)?;

                let child_props = parent_prop.child_props(ctx).await?;

                // Determine if there are extra/invalid fields in the unprocess object that have no
                // corresponding prop field
                let object_keys: HashSet<_> =
                    unprocessed_object.keys().map(|s| s.as_str()).collect();
                let prop_keys: HashSet<_> = child_props.iter().map(|prop| prop.name()).collect();
                let invalid_object_keys: HashSet<_> = object_keys.difference(&prop_keys).collect();
                if !invalid_object_keys.is_empty() {
                    return Err(AttributeValueError::InvalidObjectValueFields(
                        invalid_object_keys
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                    ));
                }

                let unset_func_name = "si:unset".to_string();
                let unset_func = Func::find_by_attr(ctx, "name", &unset_func_name)
                    .await?
                    .pop()
                    .ok_or(AttributeValueError::MissingFunc(unset_func_name))?;
                let (unset_func_binding, unset_func_binding_return_value) =
                    FuncBinding::find_or_create_and_execute(
                        ctx,
                        serde_json::json![null],
                        *unset_func.id(),
                    )
                    .await?;

                for prop in child_props {
                    // If an unprocessed object field exists, remove it and process it as a new
                    // AttributeValue
                    if let Some(value) = unprocessed_object.remove(prop.name()) {
                        let mut context_builder = AttributeContextBuilder::from(update_context);
                        let context = context_builder.set_prop_id(*prop.id()).to_context()?;

                        let maybe_attribute_value = Self::find_with_parent_and_key_for_context(
                            ctx,
                            Some(*parent_attribute_value.id()),
                            None,
                            context.into(),
                        )
                        .await?;

                        let attribute_value = match maybe_attribute_value {
                            Some(attribute_value) => attribute_value,
                            None => {
                                let attribute_value = Self::new(
                                    ctx,
                                    *unset_func_binding.id(),
                                    *unset_func_binding_return_value.id(),
                                    context,
                                    None::<&str>,
                                )
                                .await?;
                                attribute_value
                                    .set_parent_attribute_value(ctx, parent_attribute_value.id())
                                    .await?;

                                AttributePrototype::new_with_existing_value(
                                    ctx,
                                    *unset_func.id(),
                                    context,
                                    None,
                                    Some(*parent_attribute_value.id()),
                                    *attribute_value.id(),
                                )
                                .await
                                .map_err(|e| {
                                    AttributeValueError::AttributePrototype(format!("{e}"))
                                })?;

                                attribute_value
                            }
                        };

                        let (_, _) = Self::update_for_context_without_creating_proxies(
                            ctx,
                            *attribute_value.id(),
                            Some(*parent_attribute_value.id()),
                            context,
                            Some(value),
                            None,
                        )
                        .await?;
                    }
                }
            }
            PropKind::Array => {
                let unprocessed_array = unprocessed_value
                    .as_array_mut()
                    .ok_or(AttributeValueError::ValueAsObject)?;

                for value in unprocessed_array.drain(0..) {
                    let _ = Self::insert_for_context_without_creating_proxies(
                        ctx,
                        update_context,
                        *parent_attribute_value.id(),
                        Some(value),
                        None,
                    )
                    .await?;
                }
            }
            PropKind::Map => {
                let unprocessed_map = unprocessed_value
                    .as_object_mut()
                    .ok_or(AttributeValueError::ValueAsMap)?;

                let map_keys: HashSet<_> = unprocessed_map.keys().map(|s| s.to_string()).collect();
                for key in map_keys {
                    if let Some(value) = unprocessed_map.remove(&key) {
                        let _ = Self::insert_for_context_without_creating_proxies(
                            ctx,
                            update_context,
                            *parent_attribute_value.id(),
                            Some(value),
                            Some(key),
                        )
                        .await?;
                    }
                }
            }
            unexpected @ PropKind::String
            | unexpected @ PropKind::Boolean
            | unexpected @ PropKind::Integer => {
                return Err(AttributeValueError::UnexpectedPropKind(*unexpected));
            }
        };

        Ok(())
    }

    #[async_recursion]
    async fn remove_value_and_children(
        ctx: &DalContext<'_, '_>,
        parent_attribute_value: AttributeValue,
    ) -> AttributeValueResult<()> {
        let child_values = parent_attribute_value.child_attribute_values(ctx).await?;
        for child_value in child_values {
            Self::remove_value_and_children(ctx, child_value).await?;
        }

        parent_attribute_value.remove_proxies(ctx).await?;

        let attribute_prototype = parent_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or_else(|| {
                AttributeValueError::AttributePrototypeNotFound(
                    *parent_attribute_value.id(),
                    *ctx.visibility(),
                )
            })?;

        parent_attribute_value
            .unset_attribute_prototype(ctx)
            .await?;

        // If our value is the only remaining value for the prototype, delete the prototype
        if attribute_prototype
            .attribute_values(ctx)
            .await
            .map_err(|err| AttributeValueError::AttributePrototype(err.to_string()))?
            .is_empty()
        {
            attribute_prototype
                .delete(ctx)
                .await
                .map_err(|err| AttributeValueError::AttributePrototype(err.to_string()))?;
        }

        parent_attribute_value.delete(ctx).await?;

        Ok(())
    }

    async fn remove_proxies(&self, ctx: &DalContext<'_, '_>) -> AttributeValueResult<()> {
        let _row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT succeeded FROM attribute_value_remove_proxies_v1($1, $2, $3)",
                &[&ctx.write_tenancy(), ctx.visibility(), &self.id()],
            )
            .await?;
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
) -> AttributeValueResult<(Func, FuncBinding, FuncBindingReturnValue)> {
    let func_name = func_name.to_owned();
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or(AttributeValueError::MissingFunc(func_name))?;

    let (func_binding, func_binding_return_value) =
        FuncBinding::find_or_create_and_execute(ctx, args, *func.id()).await?;

    Ok((func, func_binding, func_binding_return_value))
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
