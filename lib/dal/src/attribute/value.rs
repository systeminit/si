//! An [`AttributeValue`] represents which [`FuncBinding`](crate::func::binding::FuncBinding)
//! and [`FuncBindingReturnValue`] provide attribute's value. Moreover, it tracks whether the
//! value is proxied or not. Proxied values "point" to another [`AttributeValue`] to provide
//! the attribute's value.

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute::{
        context::{
            AttributeContext, AttributeContextBuilder, AttributeContextBuilderError,
            AttributeReadContext,
        },
        prototype::{AttributePrototype, AttributePrototypeId},
    },
    func::{
        binding::{FuncBindingError, FuncBindingId},
        binding_return_value::{
            FuncBindingReturnValue, FuncBindingReturnValueError, FuncBindingReturnValueId,
        },
        FuncId,
    },
    impl_standard_model,
    job::definition::DependentValuesUpdate,
    pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    AttributeContextError, AttributePrototypeArgumentError, Component, ComponentId, DalContext,
    Func, FuncBackendKind, FuncBackendResponseType, FuncBinding, FuncError, HistoryEventError,
    IndexMap, InternalProvider, InternalProviderId, Prop, PropError, PropId, PropKind,
    ReadTenancyError, StandardModel, StandardModelError, Timestamp, TransactionsError, Visibility,
    WriteTenancy, WsEventError,
};

pub mod view;

const CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_child_attribute_values_for_context.sql");
const FETCH_UPDATE_GRAPH_DATA: &str =
    include_str!("../queries/attribute_value/fetch_update_graph_data.sql");
const IS_FOR_INTERNAL_PROVIDER_OF_ROOT_PROP: &str =
    include_str!("../queries/attribute_value/is_for_internal_provider_of_root_prop.sql");
const FIND_PROP_FOR_VALUE: &str =
    include_str!("../queries/attribute_value_find_prop_for_value.sql");
const FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_find_with_parent_and_key_for_context.sql");
const FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value_find_with_parent_and_prototype_for_context.sql");
const LIST_FOR_CONTEXT: &str = include_str!("../queries/attribute_value_list_for_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT: &str =
    include_str!("../queries/attribute_value_list_payload_for_read_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT: &str =
    include_str!("../queries/attribute_value_list_payload_for_read_context_and_root.sql");

#[derive(Error, Debug)]
pub enum AttributeValueError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
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
    #[error("No AttributeValue found for AttributeReadContext: {0:?}")]
    NotFoundForReadContext(AttributeReadContext),
    #[error("using json pointer for attribute view yielded no value")]
    NoValueForJsonPointer,
    #[error(
        "parent must be for an array, map, or object prop: attribute resolver id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributeValueId, PropKind),
    #[error("parent not found or does not exist for value: {0}")]
    ParentNotFound(AttributeValueId),
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
    #[error("component missing in context: {0:?}")]
    MissingComponentInReadContext(AttributeReadContext),
    #[error("component not found by id: {0}")]
    ComponentNotFoundById(ComponentId),
    #[error("schema variant missing in context")]
    SchemaVariantMissing,
    #[error("schema missing in context")]
    SchemaMissing,
    #[error("ws event publishing error")]
    WsEvent(#[from] WsEventError),
}

/// This is the function that set the attribute value, along with the function's prototype
/// context (and other metadata). It corresponds to the PostgreSQL type
/// "func_with_attribute_prototype_context"
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncWithPrototypeContext {
    id: FuncId,
    name: String,
    #[serde(rename(serialize = "displayName"))]
    display_name: Option<String>,
    #[serde(rename(serialize = "backendKind"))]
    backend_kind: FuncBackendKind,
    #[serde(rename(serialize = "backendResponseType"))]
    backend_response_type: FuncBackendResponseType,
    #[serde(rename(serialize = "isBuiltin"))]
    is_builtin: bool,
    #[serde(rename(serialize = "attributePrototypeId"))]
    attribute_prototype_id: AttributePrototypeId,
    #[serde(rename(serialize = "attributeContextComponentId"))]
    attribute_context_component_id: ComponentId,
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
        ctx: &DalContext,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        context: AttributeContext,
        key: Option<impl Into<String>>,
    ) -> AttributeValueResult<Self> {
        let key: Option<String> = key.map(|s| s.into());
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT new_attribute_value AS object FROM attribute_value_new_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &func_binding_id,
                    &func_binding_return_value_id,
                    &context,
                    &key,
                ],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    standard_model_accessor!(
        proxy_for_attribute_value_id,
        Option<Pk(AttributeValueId)>,
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
        ctx: &DalContext,
        belongs_to_id: &AttributeValueId,
    ) -> AttributeValueResult<()> {
        let _row = ctx
            .txns()
            .pg()
            .query(
                "SELECT attribute_value_set_parent_attribute_value_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &self.id,
                    belongs_to_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub fn index_map_mut(&mut self) -> Option<&mut IndexMap> {
        self.index_map.as_mut()
    }

    /// Returns the *unprocessed* [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
    /// corresponding to the field on [`Self`].
    pub async fn get_unprocessed_value(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
            Some(func_binding_return_value) => {
                Ok(func_binding_return_value.unprocessed_value().cloned())
            }
            None => Err(AttributeValueError::MissingFuncBindingReturnValue),
        }
    }

    /// Returns the [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
    /// corresponding to the field on [`Self`].
    pub async fn get_value(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
            Some(func_binding_return_value) => Ok(func_binding_return_value.value().cloned()),
            None => Err(AttributeValueError::MissingFuncBindingReturnValue),
        }
    }

    pub async fn update_stored_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
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
        ctx: &DalContext,
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

    pub async fn find_with_parent_and_prototype_for_context(
        ctx: &DalContext,
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
        ctx: &DalContext,
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
        ctx: &DalContext,
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
        ctx: &DalContext,
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
        ctx: &DalContext,
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

    /// List [`AttributeValuePayloads`](AttributeValuePayload) for a given
    /// [`context`](crate::AttributeReadContext), which must specify a
    /// [`ComponentId`](crate::Component).
    pub async fn list_payload_for_read_context(
        ctx: &DalContext,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
        let schema_variant_id = match context.component_id {
            Some(component_id) if component_id != ComponentId::NONE => {
                let component = Component::get_by_id(ctx, &component_id)
                    .await?
                    .ok_or(AttributeValueError::ComponentNotFoundById(component_id))?;
                let schema_variant = component
                    .schema_variant(ctx)
                    .await
                    .map_err(|e| AttributeValueError::Component(e.to_string()))?
                    .ok_or(AttributeValueError::SchemaVariantNotFoundForComponent(
                        component_id,
                    ))?;
                *schema_variant.id()
            }
            _ => {
                return Err(AttributeValueError::MissingComponentInReadContext(context));
            }
        };

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

            let func_view_json: serde_json::Value = row.try_get("func_with_prototype_context")?;
            let func_view: FuncWithPrototypeContext = serde_json::from_value(func_view_json)?;

            result.push(AttributeValuePayload::new(
                prop,
                func_binding_return_value,
                attribute_value,
                parent_attribute_value_id,
                func_view,
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
        ctx: &DalContext,
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

            let func_view_json: serde_json::Value = row.try_get("func_with_prototype_context")?;
            let func_view: FuncWithPrototypeContext = serde_json::from_value(func_view_json)?;

            result.push(AttributeValuePayload::new(
                prop,
                func_binding_return_value,
                attribute_value,
                parent_attribute_value_id,
                func_view,
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
        ctx: &DalContext,
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
        ctx: &DalContext,
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
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        context: AttributeContext,
        value: Option<serde_json::Value>,
        // TODO: Allow updating the key
        key: Option<String>,
        create_child_proxies: bool,
    ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
        let row = ctx.pg_txn().query_one(
            "SELECT new_attribute_value_id FROM attribute_value_update_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                ctx.write_tenancy(),
                ctx.read_tenancy(),
                ctx.visibility(),
                &attribute_value_id,
                &parent_attribute_value_id,
                &context,
                &value,
                &key,
                &create_child_proxies,
            ],
            ).await?;

        let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

        // TODO(fnichol): we might want to fire off a status even at this point, however we've
        // already updated the initial attribute value, so is there much value?

        ctx.enqueue_job(DependentValuesUpdate::new(ctx, new_attribute_value_id))
            .await;

        Ok((value, new_attribute_value_id))
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
        ctx: &DalContext,
        item_attribute_context: AttributeContext,
        array_or_map_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        Self::insert_for_context_raw(
            ctx,
            item_attribute_context,
            array_or_map_attribute_value_id,
            value,
            key,
            true,
        )
        .await
    }

    #[instrument(skip_all)]
    pub async fn insert_for_context_without_creating_proxies(
        ctx: &DalContext,
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
        ctx: &DalContext,
        item_attribute_context: AttributeContext,
        array_or_map_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
        create_child_proxies: bool,
    ) -> AttributeValueResult<AttributeValueId> {
        let row = ctx.pg_txn().query_one(
            "SELECT new_attribute_value_id FROM attribute_value_insert_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                ctx.write_tenancy(),
                ctx.read_tenancy(),
                ctx.visibility(),
                &item_attribute_context,
                &array_or_map_attribute_value_id,
                &value,
                &key,
                &create_child_proxies,
            ],
        ).await?;

        let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

        ctx.enqueue_job(DependentValuesUpdate::new(ctx, new_attribute_value_id))
            .await;

        Ok(new_attribute_value_id)
    }

    #[instrument(skip_all)]
    pub async fn update_parent_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
        let _row = ctx
            .pg_txn()
            .query(
                "SELECT attribute_value_update_parent_index_map_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &self.id,
                ],
            )
            .await?;

        Ok(())
    }

    async fn populate_nested_values(
        ctx: &DalContext,
        parent_attribute_value_id: AttributeValueId,
        update_context: AttributeContext,
        unprocessed_value: serde_json::Value,
    ) -> AttributeValueResult<()> {
        let _row = ctx
            .pg_txn()
            .query(
                "SELECT attribute_value_populate_nested_values_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.write_tenancy(),
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &parent_attribute_value_id,
                    &update_context,
                    &unprocessed_value,
                ],
            )
            .await?;

        Ok(())
    }

    /// Convenience method to determine if this [`AttributeValue`](Self) is for the implicit
    /// [`InternalProvider`](crate::InternalProvider) that represents the "snapshot" of the entire
    /// [`Component`](crate::Component). This means that the [`Prop`](crate::Prop) that the
    /// [`InternalProvider`](crate::InternalProvider) is sourcing its data from does not have a
    /// parent [`Prop`](crate::Prop).
    #[allow(unused)]
    async fn is_for_internal_provider_of_root_prop(
        &mut self,
        ctx: &DalContext,
    ) -> AttributeValueResult<bool> {
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(
                IS_FOR_INTERNAL_PROVIDER_OF_ROOT_PROP,
                &[&ctx.write_tenancy(), ctx.visibility(), &self.context],
            )
            .await?;
        if let Some(row) = maybe_row {
            // If we got a row back, that means that we are an AttributeValue for an InternalProvider,
            // and we should have gotten a row back from the query.
            Ok(row.try_get("is_for_root_prop")?)
        } else {
            // If we didn't get a row back, that means that we didn't find an InternalProvider for the
            // InternalProviderId in our AttributeContext. Likely because it is ident_nil_v1, indicating that we're
            // not for an InternalProvider at all.
            Ok(false)
        }
    }

    /// Returns a `HashMap` of `AttributeValueId -> Vec<AttributeValueId>` where the keys
    /// are `AttributeValue`s that are affected (directly, and indirectly) by `self`
    /// having a new value. The `Vec<AttributeValueId>` are the `AttributeValue` that the
    /// key directly depends on that are also affected by `self` having a new value.
    ///
    /// **NOTE**: This has the side effect of **CREATING NEW [`AttributeValue`s][AttributeValue]**
    /// if this [`AttributeValue`][AttributeValue] affects an [`AttributeContext`][AttributeContext]
    /// where an [`AttributePrototype`][AttributePrototype] that uses it didn't already have an
    /// [`AttributeValue`][AttributeValue].
    pub async fn dependent_value_graph(
        &mut self,
        ctx: &DalContext,
    ) -> AttributeValueResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
        info!("AttributeValue.dependent_value_graph(): {:?}", &self);
        let total_start = std::time::Instant::now();

        let _rows = ctx
            .pg_txn()
            .query(
                "SELECT attribute_value_create_new_affected_values_v1($1, $2, $3, $4)",
                &[
                    &ctx.write_tenancy(),
                    &ctx.read_tenancy(),
                    &ctx.visibility(),
                    &self.id,
                ],
            )
            .await?;
        info!(
            "AttributeValue.dependent_value_graph(): Create new affected took {:?}",
            total_start.elapsed()
        );
        let section_start = std::time::Instant::now();

        let rows = ctx
            .txns()
            .pg()
            .query(
                FETCH_UPDATE_GRAPH_DATA,
                &[&ctx.read_tenancy(), ctx.visibility(), &self.id],
            )
            .await?;
        info!(
            "AttributeValue.dependent_value_graph(): Graph query took {:?} (total elapsed {:?}",
            section_start.elapsed(),
            total_start.elapsed(),
        );

        let mut result: HashMap<AttributeValueId, Vec<AttributeValueId>> = HashMap::new();
        for row in rows.into_iter() {
            let attr_val_id: AttributeValueId = row.try_get("attribute_value_id")?;
            let dependencies: Vec<AttributeValueId> =
                row.try_get("dependent_attribute_value_ids")?;
            result.insert(attr_val_id, dependencies);
        }

        info!(
            "AttributeValue.dependent_value_graph(): Total elapsed {:?}",
            total_start.elapsed()
        );

        Ok(result)
    }

    pub async fn vivify_value_and_parent_values(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<AttributeValueId> {
        let row = ctx.pg_txn().query_one(
            "SELECT new_attribute_value_id FROM attribute_value_vivify_value_and_parent_values_raw_v1($1, $2, $3, $4, $5, $6)",
        &[
            ctx.write_tenancy(),
            ctx.read_tenancy(),
            ctx.visibility(),
            &self.context,
            &self.id,
            &true
        ]).await?;

        Ok(row.try_get("new_attribute_value_id")?)
    }

    /// Re-evaluates the current `AttributeValue`'s `AttributePrototype` to update the
    /// `FuncBinding`, and `FuncBindingReturnValue`, reflecting the current inputs to
    /// the function.
    ///
    /// If the `AttributeValue` represents the `InternalProvider` for a `Prop` that
    /// does not have a parent `Prop` (this is typically the `InternalProvider` for
    /// the "root" `Prop` of a `SchemaVariant`), then it will also enqueue a
    /// `CodeGeneration` job for the `Component`.
    pub async fn update_from_prototype_function(
        &mut self,
        ctx: &DalContext,
    ) -> AttributeValueResult<()> {
        let update_start_time = std::time::Instant::now();

        // Check if this AttributeValue is for an implicit InternalProvider as they have special behavior that doesn't involve
        // AttributePrototype and AttributePrototypeArguments.
        if self
            .context
            .is_least_specific_field_kind_internal_provider()?
        {
            let internal_provider =
                InternalProvider::get_by_id(ctx, &self.context.internal_provider_id())
                    .await?
                    .ok_or_else(|| {
                        AttributeValueError::InternalProviderNotFound(
                            self.context.internal_provider_id(),
                        )
                    })?;
            if internal_provider.is_internal_consumer() {
                // We don't care about the AttributeValue that comes back from implicit_emit, since we should already be
                // operating on an AttributeValue that has the correct AttributeContext, which means that a new one should
                // not need to be created.
                internal_provider
                    .implicit_emit(ctx, self)
                    .await
                    .map_err(|e| AttributeValueError::InternalProvider(e.to_string()))?;

                info!(
                        "update_from_prototype_function({:?}) took {:?} InternalProvider is internal consumer",
                        self.id,
                        update_start_time.elapsed()
                    );

                return Ok(());
            }
        } else if self.context.is_least_specific_field_kind_prop()? {
            if let Some(parent_attribute_value) = self.parent_attribute_value(ctx).await? {
                parent_attribute_value
                    .vivify_value_and_parent_values(ctx)
                    .await?;
            }
        }

        // The following should handle explicit "normal" Attributes, InternalProviders, and ExternalProviders already.
        let attribute_prototype = self.attribute_prototype(ctx).await?.ok_or_else(|| {
            AttributeValueError::AttributePrototypeNotFound(self.id, *ctx.visibility())
        })?;

        let mut func_binding_args: HashMap<String, Option<serde_json::Value>> = HashMap::new();
        for mut argument_data in attribute_prototype
            .argument_values(ctx, self.context)
            .await
            .map_err(|e| AttributeValueError::AttributePrototype(e.to_string()))?
        {
            match argument_data.values.len() {
                1 => {
                    let argument = argument_data.values.pop().ok_or_else(|| {
                        AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
                            argument_data.argument_name.clone(),
                        )
                    })?;

                    func_binding_args.insert(
                        argument_data.argument_name,
                        Some(serde_json::to_value(argument)?),
                    );
                }
                2.. => {
                    func_binding_args.insert(
                        argument_data.argument_name,
                        Some(serde_json::to_value(argument_data.values)?),
                    );
                }
                _ => {
                    return Err(
                        AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
                            argument_data.argument_name,
                        ),
                    )
                }
            };
        }

        // If a function has no input, we probably don't need to re-execute it.
        // TODO: we should confirm from a product standpoint whether this is true!
        // However, if the function code has changed since the last execution, we *should*
        // re-execute it, since it's a different function now
        if func_binding_args.is_empty() {
            let func = Func::get_by_id(ctx, &attribute_prototype.func_id())
                .await?
                .ok_or_else(|| {
                    AttributeValueError::MissingFunc(
                        "Missing attribute prototype function".to_string(),
                    )
                })?;

            if let Some(func_binding) = FuncBinding::get_by_id(ctx, &self.func_binding_id()).await?
            {
                // Restricting the re-execution of no-args functions to
                // JsAttribute funcs for now (unless code has changed!) This
                // breaks inter_component_identity_update tests otherwise, but
                // I'm not sure why.
                if (func_binding.args() == &serde_json::json!({})
                    || func_binding.backend_kind() != &FuncBackendKind::JsAttribute)
                    && func_binding.code_sha256() == func.code_sha256()
                {
                    return Ok(());
                }
            }
        }

        let execution_start = std::time::Instant::now();
        let (func_binding, mut func_binding_return_value) = FuncBinding::create_and_execute(
            ctx,
            serde_json::to_value(func_binding_args)?,
            attribute_prototype.func_id(),
        )
        .await?;

        info!(
            "update_from_prototype_function({:?}): Func {:?} execution took {:?}",
            self.id,
            attribute_prototype.func_id(),
            execution_start.elapsed()
        );

        self.set_func_binding_id(ctx, *func_binding.id()).await?;
        self.set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
            .await?;

        // If the value we just updated was for a Prop, we might have run a function that
        // generates a deep data structure. If the Prop is an Array/Map/Object, then the
        // value should be an empty Array/Map/Object, while the unprocessed value contains
        // the deep data structure.
        if self.context.is_least_specific_field_kind_prop()? {
            let processed_value = match func_binding_return_value.unprocessed_value().cloned() {
                Some(unprocessed_value) => {
                    let prop = Prop::get_by_id(ctx, &self.context.prop_id())
                        .await?
                        .ok_or_else(|| AttributeValueError::PropNotFound(self.context.prop_id()))?;

                    match prop.kind() {
                        PropKind::Object | PropKind::Map => Some(serde_json::json!({})),
                        PropKind::Array => Some(serde_json::json!([])),
                        _ => Some(unprocessed_value),
                    }
                }
                None => None,
            };

            // This check prevents writing to a func_binding_return_value that has already been processed
            // The reason we need that is because sometimes we get a fbrv that was processed at the universal
            // tenancy, so it tries to write to universal tenancy and fails. This happens mostly to identity
            // with "" as args.
            if func_binding_return_value.value() != processed_value.as_ref() {
                func_binding_return_value
                    .set_value(ctx, processed_value)
                    .await?;
            }
        };
        // The value will be different from the unprocessed value if we updated it above
        // for an Array/Map/Value. If they are different from each other, then we know
        // that we need to fully process the deep data structure, populating
        // AttributeValues for the child Props.
        if func_binding_return_value.unprocessed_value() != func_binding_return_value.value() {
            if let Some(unprocessed_value) = func_binding_return_value.unprocessed_value().cloned()
            {
                AttributeValue::populate_nested_values(
                    ctx,
                    self.id,
                    self.context,
                    unprocessed_value,
                )
                .await?;
            }
        }

        info!(
            "update_from_prototype_function({:?}) took {:?}",
            self.id,
            update_start_time.elapsed()
        );

        Ok(())
    }
}

#[derive(Debug)]
pub struct AttributeValuePayload {
    pub prop: Prop,
    pub func_binding_return_value: Option<FuncBindingReturnValue>,
    pub attribute_value: AttributeValue,
    pub parent_attribute_value_id: Option<AttributeValueId>,
    pub func_with_prototype_context: FuncWithPrototypeContext,
}

impl AttributeValuePayload {
    pub fn new(
        prop: Prop,
        func_binding_return_value: Option<FuncBindingReturnValue>,
        attribute_value: AttributeValue,
        parent_attribute_value_id: Option<AttributeValueId>,
        func_with_prototype_context: FuncWithPrototypeContext,
    ) -> Self {
        Self {
            prop,
            func_binding_return_value,
            attribute_value,
            parent_attribute_value_id,
            func_with_prototype_context,
        }
    }
}
