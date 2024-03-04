//! An [`AttributeValue`] represents which [`FuncBinding`](crate::func::binding::FuncBinding)
//! and [`FuncBindingReturnValue`] provide attribute's value. Moreover, it tracks whether the
//! value is proxied or not. Proxied values "point" to another [`AttributeValue`] to provide
//! the attribute's value.
//!
//! ## Updating [`AttributeValues`](AttributeValue)
//!
//! Let's say you want to update a
//! [`PropertyEditorValue`](crate::property_editor::values::PropertyEditorValue) in the UI or a
//! "field" on a [`Component`](crate::Component) in general. The key to doing so is the following
//! process:
//!
//! 1) Find the appropriate [`AttributeValue`] in a [`context`](crate::AttributeContext) that is
//!   either "exactly specific" to what you need or "less specific" than what you need (see the
//!   [`module`](crate::attribute::context) for more information)
//! 2) Find its parent, which almost all [`AttributeValues`](AttributeValue) should have if they are
//!   in the lineage of a [`RootProp`](crate::RootProp) (usually, the
//!   [`standard model accessor`](crate::standard_accessors) that contains the parent will suffice
//!   in finding the parent)
//! 3) Use [`AttributeValue::update_for_context()`] with the appropriate key and
//!   [`context`](crate::AttributeContext) while ensuring that if you reuse the key and/or
//!   [`context`](crate::AttributeContext) from the [`AttributeValue`](crate::AttributeValue)
//!   that you found, that it is _exactly_ what you need (i.e. if the key changes or the
//!   [`context`](crate::AttributeContext) is in a lesser specificity than what you need, you
//!   mutate them accordingly)
//!
//! Often, you may not have all the information necessary to find the [`AttributeValue`] that you
//! would like to update. Ideally, you would use one of the existing accessor methods off
//! [`AttributeValue`] with contextual information such as a [`PropId`](crate::Prop),
//! a [`ComponentId`](crate::Component)), a parent [`AttributeValue`], a key, etc.
//!
//! In situations where we do not have minimal information to find the _correct_ [`AttributeValue`]
//! from existing accessor queries, we can leveraging existing queries from other structs and write
//! new queries for those structs and specific use cases. For example, since most members of the
//! [`RootProp`](crate::RootProp) tree are stable across [`SchemaVariants`](crate::SchemaVariant),
//! we can use [`Component::root_prop_child_attribute_value_for_component()`](crate::Component::root_prop_child_attribute_value_for_component)
//! to find the [`AttributeValue`] whose [`context`](crate::AttributeContext) corresponds to a
//! direct child [`Prop`](crate::Prop) of the [`RootProp`](crate::RootProp).

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::before::before_funcs_for_component;
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
    },
    impl_standard_model, pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    AttributeContextError, AttributePrototypeArgumentError, Component, ComponentId, DalContext,
    Func, FuncBinding, FuncError, HistoryEventError, IndexMap, InternalProvider,
    InternalProviderId, Prop, PropError, PropId, PropKind, StandardModel, StandardModelError,
    Tenancy, Timestamp, TransactionsError, Visibility, WsEventError,
};
use crate::{ExternalProviderId, FuncId};

pub mod view;

const ATTRIBUTE_VALUE_IDS_FOR_COMPONENT: &str =
    include_str!("../queries/attribute_value/ids_for_component.sql");
const ATTRIBUTE_VALUE_IDS_WITH_DYNAMIC_FUNCTIONS: &str =
    include_str!("../queries/attribute_value/ids_with_dynamic_functions.sql");
const CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value/child_attribute_values_for_context.sql");
const FETCH_UPDATE_GRAPH_DATA: &str =
    include_str!("../queries/attribute_value/fetch_update_graph_data.sql");
const FIND_PROP_FOR_VALUE: &str =
    include_str!("../queries/attribute_value/find_prop_for_value.sql");
const FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value/find_with_parent_and_key_for_context.sql");
const FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_value/find_with_parent_and_prototype_for_context.sql");
const LIST_FOR_CONTEXT: &str = include_str!("../queries/attribute_value/list_for_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT: &str =
    include_str!("../queries/attribute_value/list_payload_for_read_context.sql");
const LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT: &str =
    include_str!("../queries/attribute_value/list_payload_for_read_context_and_root.sql");
const FIND_CONTROLLING_FUNCS: &str =
    include_str!("../queries/attribute_value/find_controlling_funcs.sql");
const LIST_ATTRIBUTES_WITH_OVERRIDDEN: &str =
    include_str!("../queries/attribute_value/list_attributes_with_overridden.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributeValueError {
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
    #[error("component not found by id: {0}")]
    ComponentNotFoundById(ComponentId),
    #[error(transparent)]
    Council(#[from] council_server::client::ClientError),
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
    #[error("function result failure: kind={kind}, message={message}, backend={backend}")]
    FuncBackendResultFailure {
        kind: String,
        message: String,
        backend: String,
    },
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
    #[error("found invalid object value fields not found in corresponding prop: {0:?}")]
    InvalidObjectValueFields(Vec<String>),
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
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
    #[error("component missing in context: {0:?}")]
    MissingComponentInReadContext(AttributeReadContext),
    #[error("missing attribute value with id: {0}")]
    MissingForId(AttributeValueId),
    #[error("func not found: {0}")]
    MissingFunc(String),
    #[error("FuncBinding not found: {0}")]
    MissingFuncBinding(FuncBindingId),
    #[error("func binding return value not found")]
    MissingFuncBindingReturnValue,
    #[error("func information not found for attribute value id: {0}")]
    MissingFuncInformation(AttributeValueId),
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
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("Prop not found: {0}")]
    PropNotFound(PropId),
    #[error("schema missing in context")]
    SchemaMissing,
    #[error("schema not found for component id: {0}")]
    SchemaNotFoundForComponent(ComponentId),
    #[error("schema variant missing in context")]
    SchemaVariantMissing,
    #[error("schema variant not found for component id: {0}")]
    SchemaVariantNotFoundForComponent(ComponentId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
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
    #[error("ws event publishing error")]
    WsEvent(#[from] WsEventError),
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

pub struct ComponentValuePayload {
    pub attribute_value: AttributeValue,
    pub maybe_parent_attribute_value_id: Option<AttributeValueId>,
}

impl AttributeValue {
    #[instrument(level = "debug", skip(ctx, key), fields(key))]
    pub async fn new(
        ctx: &DalContext,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        context: AttributeContext,
        key: Option<impl Into<String>>,
    ) -> AttributeValueResult<Self> {
        let key: Option<String> = key.map(|s| s.into());
        tracing::Span::current().record("key", &key);
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT new_attribute_value AS object FROM attribute_value_new_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
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

    // pub async fn save_index_map(
    //     &self,
    //     ctx: &DalContext,
    //     index_map: IndexMap,
    // ) -> AttributeValueResult<()> {
    // }

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
            .txns()
            .await?
            .pg()
            .query(
                CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
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
            .await?
            .pg()
            .query_opt(
                FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context,
                    &attribute_prototype_id,
                    &parent_attribute_value_id,
                ],
            )
            .await?;

        Ok(standard_model::option_object_from_row(row)?)
    }

    pub async fn find_all_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> AttributeValueResult<Vec<ComponentValuePayload>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT DISTINCT ON (av.id)
                    row_to_json(av.*) AS av_object,
                    avbtav.belongs_to_id AS parent_attribute_value_id
            FROM attribute_values_v1($1, $2) AS av
            LEFT JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) as avbtav
                ON av.id = avbtav.object_id
            WHERE attribute_context_component_id = $3",
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?;

        let mut result = vec![];
        for row in rows {
            let av_json: serde_json::Value = row.try_get("av_object")?;
            let attribute_value: Self = serde_json::from_value(av_json)?;

            let maybe_parent_attribute_value_id: Option<AttributeValueId> =
                row.try_get("parent_attribute_value_id")?;

            result.push(ComponentValuePayload {
                attribute_value,
                maybe_parent_attribute_value_id,
            });
        }

        Ok(result)
    }

    /// Find [`Self`] with a given parent value and key.
    pub async fn find_with_parent_and_key_for_context(
        ctx: &DalContext,
        parent_attribute_value_id: Option<AttributeValueId>,
        key: Option<String>,
        context: AttributeReadContext,
    ) -> AttributeValueResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
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
            .await?
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[ctx.tenancy(), ctx.visibility(), &context],
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
            .await?
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[ctx.tenancy(), ctx.visibility(), &context],
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
            .await?
            .pg()
            .query_one(
                FIND_PROP_FOR_VALUE,
                &[ctx.tenancy(), ctx.visibility(), &attribute_value_id],
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
                // We get the component even if it gets deleted because we may still need to operate with
                // attribute values of soft deleted components
                let component =
                    Component::get_by_id(&ctx.clone_with_delete_visibility(), &component_id)
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
            .await?
            .pg()
            .query(
                LIST_PAYLOAD_FOR_READ_CONTEXT,
                &[
                    ctx.tenancy(),
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
            .await?
            .pg()
            .query(
                LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT,
                &[
                    ctx.tenancy(),
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

    // Eventually, this should be usable for *ALL* Component AttributeValues, but
    // there isn't much point in supporting not-Prop AttributeValues until there
    // is a way to assign functions other than the identity function to them.
    pub async fn use_default_prototype(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT attribute_value_use_default_prototype_v1($1, $2, $3) AS changed",
                &[ctx.tenancy(), ctx.visibility(), &attribute_value_id],
            )
            .await?;

        if row.get("changed") {
            // Update from prototype & trigger dependent values update
            let mut av = AttributeValue::get_by_id(ctx, &attribute_value_id)
                .await?
                .ok_or_else(|| {
                    AttributeValueError::NotFound(attribute_value_id, *ctx.visibility())
                })?;
            av.update_from_prototype_function(ctx).await?;
            ctx.enqueue_dependent_values_update(vec![attribute_value_id])
                .await?;
        }

        Ok(())
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
            true,
        )
        .await
    }

    pub async fn update_for_context_without_propagating_dependent_values(
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
            false,
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
            true,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_for_context_raw(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        context: AttributeContext,
        value: Option<serde_json::Value>,
        // TODO: Allow updating the key
        key: Option<String>,
        create_child_proxies: bool,
        propagate_dependent_values: bool,
    ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
        // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to do this in the future.
        let ctx = &ctx.clone_without_deleted_visibility();

        let row = ctx.txns()
            .await?
            .pg()
            .query_one(
                "SELECT new_attribute_value_id FROM attribute_value_update_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
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

        if !context.is_component_unset() {
            ctx.enqueue_dependencies_update_component(context.component_id())
                .await?;
        }

        // TODO(fnichol): we might want to fire off a status even at this point, however we've
        // already updated the initial attribute value, so is there much value?

        if propagate_dependent_values && !ctx.no_dependent_values() {
            ctx.enqueue_dependent_values_update(vec![new_attribute_value_id])
                .await?;
        }

        if let Some(av) = AttributeValue::get_by_id(ctx, &new_attribute_value_id).await? {
            Prop::run_validation(
                ctx,
                context.prop_id(),
                context.component_id(),
                av.key(),
                value.clone().unwrap_or_default(),
            )
            .await;
        }

        Ok((value, new_attribute_value_id))
    }

    /// Insert a new value under the parent [`AttributeValue`] in the given [`AttributeContext`]. This is mostly only
    /// useful for adding elements to a [`PropKind::Array`], or to a [`PropKind::Map`]. Updating existing values in an
    /// [`Array`](PropKind::Array), or [`Map`](PropKind::Map), and setting/updating all other [`PropKind`] should be
    /// able to directly use [`update_for_context()`](AttributeValue::update_for_context()), as there will already be an
    /// appropriate [`AttributeValue`] to use. By using this function,
    /// [`update_for_context()`](AttributeValue::update_for_context()) is called after we have created an appropriate
    /// [`AttributeValue`] to use.
    #[instrument(skip_all, level = "debug")]
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

    #[instrument(skip_all, level = "debug")]
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

    #[instrument(skip_all, level = "debug")]
    async fn insert_for_context_raw(
        ctx: &DalContext,
        item_attribute_context: AttributeContext,
        array_or_map_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
        create_child_proxies: bool,
    ) -> AttributeValueResult<AttributeValueId> {
        let row = ctx.txns().await?.pg().query_one(
            "SELECT new_attribute_value_id FROM attribute_value_insert_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &item_attribute_context,
                &array_or_map_attribute_value_id,
                &value,
                &key,
                &create_child_proxies,
            ],
        ).await?;

        let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

        if !item_attribute_context.is_component_unset() {
            ctx.enqueue_dependencies_update_component(item_attribute_context.component_id())
                .await?;
        }

        if !ctx.no_dependent_values() {
            ctx.enqueue_dependent_values_update(vec![new_attribute_value_id])
                .await?;
        }

        if let Some(av) = AttributeValue::get_by_id(ctx, &new_attribute_value_id).await? {
            Prop::run_validation(
                ctx,
                av.context.prop_id(),
                av.context.component_id(),
                av.key(),
                value.clone().unwrap_or_default(),
            )
            .await;
        }

        Ok(new_attribute_value_id)
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn update_parent_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
        let _row = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT attribute_value_update_parent_index_map_v1($1, $2, $3)",
                &[ctx.tenancy(), ctx.visibility(), &self.id],
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
            .txns()
            .await?
            .pg()
            .query(
                "SELECT attribute_value_populate_nested_values_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &parent_attribute_value_id,
                    &update_context,
                    &unprocessed_value,
                ],
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(ctx), level = "debug")]
    pub async fn create_dependent_values(
        ctx: &DalContext,
        attribute_value_ids: &[AttributeValueId],
    ) -> AttributeValueResult<()> {
        ctx.txns()
            .await?
            .pg()
            .execute(
                "SELECT attribute_value_create_new_affected_values_v1($1, $2, $3)",
                &[&ctx.tenancy(), &ctx.visibility(), &attribute_value_ids],
            )
            .await?;
        Ok(())
    }

    /// Returns a [`HashMap`] with key [`AttributeValueId`](Self) and value
    /// [`Vec<AttributeValueId>`](Self) where the keys correspond to [`AttributeValues`](Self) that
    /// are affected (directly and indirectly) by at least one of the provided
    /// [`AttributeValueIds`](Self) having a new value. The [`Vec<AttributeValueId>`](Self)
    /// correspond to the [`AttributeValues`](Self) that the key directly depends on that are also
    /// affected by at least one of the provided [`AttributeValueIds`](Self) having a new value.
    ///
    /// **NOTE**: This has the side effect of **CREATING NEW [`AttributeValues`](Self)**
    /// if this [`AttributeValue`] affects an [`AttributeContext`](crate::AttributeContext) where an
    /// [`AttributePrototype`](crate::AttributePrototype) that uses it didn't already have an
    /// [`AttributeValue`].
    #[instrument(skip(ctx), level = "debug")]
    pub async fn dependent_value_graph(
        ctx: &DalContext,
        attribute_value_ids: &[AttributeValueId],
    ) -> AttributeValueResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FETCH_UPDATE_GRAPH_DATA,
                &[&ctx.tenancy(), ctx.visibility(), &attribute_value_ids],
            )
            .await?;

        let mut result: HashMap<AttributeValueId, Vec<AttributeValueId>> = HashMap::new();
        for row in rows.into_iter() {
            let attr_val_id: AttributeValueId = row.try_get("attribute_value_id")?;
            let dependencies: Vec<AttributeValueId> =
                row.try_get("dependent_attribute_value_ids")?;
            result.insert(attr_val_id, dependencies);
        }

        Ok(result)
    }

    #[instrument(level = "info", skip_all)]
    pub async fn ids_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let result = ctx
            .txns()
            .await?
            .pg()
            .query(
                ATTRIBUTE_VALUE_IDS_FOR_COMPONENT,
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?
            .iter()
            .map(|r| r.get("attribute_value_id"))
            .collect();

        Ok(result)
    }

    pub async fn ids_using_dynamic_functions(
        ctx: &DalContext,
        attribute_value_ids: &Vec<AttributeValueId>,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let result = ctx
            .txns()
            .await?
            .pg()
            .query(
                ATTRIBUTE_VALUE_IDS_WITH_DYNAMIC_FUNCTIONS,
                &[ctx.tenancy(), ctx.visibility(), attribute_value_ids],
            )
            .await?
            .iter()
            .map(|r| r.get("attribute_value_id"))
            .collect();

        Ok(result)
    }

    pub async fn vivify_value_and_parent_values(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<AttributeValueId> {
        let row = ctx.txns().await?.pg().query_one(
            "SELECT new_attribute_value_id FROM attribute_value_vivify_value_and_parent_values_raw_v1($1, $2, $3, $4, $5)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &self.context,
                &self.id,
                &true
            ]).await?;

        Ok(row.try_get("new_attribute_value_id")?)
    }

    #[instrument(
        name = "attribute_value.update_component_dependencies",
        skip(ctx),
        level = "debug"
    )]
    pub async fn update_component_dependencies(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> AttributeValueResult<()> {
        ctx.txns()
            .await?
            .pg()
            .execute(
                "SELECT attribute_value_dependencies_update_component_v1($1, $2, $3)",
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?;

        Ok(())
    }

    /// Re-evaluates the current `AttributeValue`'s `AttributePrototype` to update the
    /// `FuncBinding`, and `FuncBindingReturnValue`, reflecting the current inputs to
    /// the function.
    #[instrument(
        name = "attribute_value.update_from_prototype_function",
        skip_all,
        level = "debug",
        fields(
            attribute_value.id = % self.id,
            change_set_pk = % ctx.visibility().change_set_pk,
        )
    )]
    pub async fn update_from_prototype_function(
        &mut self,
        ctx: &DalContext,
    ) -> AttributeValueResult<()> {
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

        // Check if the function is one of the "si:set*", or "si:unset" functions, as these are
        // special, and can't actually be re-run. Their values are static anyway, so re-running it
        // wouldn't change anything. The "si:setObject", "si:setArray", and "si:setMap" functions
        // are a bit special, however, as the "local" value will always be an empty object, array,
        // or map.
        let func = Func::get_by_id(ctx, &attribute_prototype.func_id())
            .await?
            .ok_or_else(|| {
                AttributeValueError::MissingFunc(format!("Unable to get func for {:?}", self.id()))
            })?;
        if func.name() == "si:setObject"
            || func.name() == "si:setMap"
            || func.name() == "si:setArray"
            || func.name() == "si:setString"
            || func.name() == "si:setInteger"
            || func.name() == "si:setBoolean"
            || func.name() == "si:unset"
        {
            return Ok(());
        }

        // Note(victor): Secrets should never be passed to functions as arguments directly.
        // We detect if they're set as dependencies and later fetch before functions to execute
        // This is so secret values still trigger the dependent values system,
        // and before functions are only called when necessary
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
                    );
                }
            };
        }

        // We need the associated [`ComponentId`] for this function--this is how we resolve and
        // prepare before functions
        let associated_component_id = self.context.component_id();
        let before = before_funcs_for_component(ctx, &associated_component_id).await?;

        let (func_binding, mut func_binding_return_value) = match FuncBinding::create_and_execute(
            ctx,
            serde_json::to_value(func_binding_args.clone())?,
            *func.id(),
            before,
        )
        .instrument(debug_span!(
            "Func execution",
            "func.id" = %func.id(),
            ?func_binding_args,
        ))
        .await
        {
            Ok(function_return_value) => function_return_value,
            Err(FuncBindingError::FuncBackendResultFailure {
                kind,
                message,
                backend,
            }) => {
                return Err(AttributeValueError::FuncBackendResultFailure {
                    kind,
                    message,
                    backend,
                });
            }
            Err(err) => Err(err)?,
        };

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

            func_binding_return_value
                .set_value(ctx, processed_value)
                .await?;
        };
        // If they are different from each other, then we know
        // that we need to fully process the deep data structure, populating
        // AttributeValues for the child Props.
        // cannot be si:setArray / si:setMap / si:setObject
        if self.context.prop_id() != PropId::NONE {
            let prop = Prop::get_by_id(ctx, &self.context.prop_id())
                .await?
                .ok_or_else(|| AttributeValueError::PropNotFound(self.context.prop_id()))?;

            if *prop.kind() == PropKind::Array
                || *prop.kind() == PropKind::Object
                || *prop.kind() == PropKind::Map
            {
                let func_name = match *prop.kind() {
                    PropKind::Array => "si:setArray",
                    PropKind::Object => "si:setObject",
                    PropKind::Map => "si:setMap",
                    _ => unreachable!(),
                };

                let func = Func::find_by_attr(ctx, "name", &func_name)
                    .await?
                    .pop()
                    .ok_or_else(|| AttributeValueError::MissingFunc(func_name.to_owned()))?;

                if attribute_prototype.func_id() != *func.id() {
                    if let Some(unprocessed_value) =
                        func_binding_return_value.unprocessed_value().cloned()
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
            }
        }

        Ok(())
    }

    pub async fn populate_child_proxies_for_value(
        &self,
        ctx: &DalContext,
        less_specific_attribute_value_id: AttributeValueId,
        more_specific_context: AttributeContext,
    ) -> AttributeValueResult<Option<Vec<AttributeValueId>>> {
        let row = ctx.txns().await?.pg().query_one(
            "SELECT new_proxy_value_ids FROM attribute_value_populate_child_proxies_for_value_v1($1, $2, $3, $4, $5)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &less_specific_attribute_value_id,
                &more_specific_context,
                self.id(),
            ],
        ).await?;

        // Are we part of a map or array? Be sure to update the index map
        if self.key.is_some() {
            ctx.txns()
                .await?
                .pg()
                .query_opt(
                    "SELECT * FROM attribute_value_update_parent_index_map_v1($1, $2, $3)",
                    &[ctx.tenancy(), ctx.visibility(), self.id()],
                )
                .await?;
        }

        Ok(row.try_get("new_proxy_value_ids")?)
    }

    /// Get the controlling function id for a particular attribute value by it's id
    /// This function id may be for a function on a parent of the attribute value
    pub async fn get_controlling_func_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> AttributeValueResult<HashMap<AttributeValueId, (FuncId, AttributeValueId, String)>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_CONTROLLING_FUNCS,
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?;

        #[derive(Clone, Debug, Deserialize)]
        struct FuncInfo {
            func_id: FuncId,
            func_name: String,
            attribute_value_id: AttributeValueId,
            parent_av_ids: Vec<AttributeValueId>,
        }

        let func_infos: Vec<FuncInfo> = standard_model::objects_from_rows(rows)?;
        let func_info_by_attribute_value_id: HashMap<AttributeValueId, FuncInfo> = func_infos
            .iter()
            .map(|info| (info.attribute_value_id, info.clone()))
            .collect();
        let mut result = HashMap::new();

        for (attribute_value_id, func_info) in &func_info_by_attribute_value_id {
            let mut ancestor_func_info = func_info.clone();
            // The parent AV IDs are populated root -> leaf, but we're most interested
            // in walking them leaf -> root.
            let mut parent_av_ids = func_info.parent_av_ids.clone();
            parent_av_ids.reverse();
            for ancestor_av_id in parent_av_ids {
                if let Some(parent_func_info) = func_info_by_attribute_value_id.get(&ancestor_av_id)
                {
                    if !(parent_func_info.func_name == "si:setObject"
                        || parent_func_info.func_name == "si:setMap"
                        || parent_func_info.func_name == "si:setArray"
                        || parent_func_info.func_name == "si:setString"
                        || parent_func_info.func_name == "si:setInteger"
                        || parent_func_info.func_name == "si:setBoolean"
                        || parent_func_info.func_name == "si:unset")
                    {
                        ancestor_func_info = parent_func_info.clone();
                        break;
                    }
                }
            }
            result.insert(
                *attribute_value_id,
                (
                    ancestor_func_info.func_id,
                    ancestor_func_info.attribute_value_id,
                    ancestor_func_info.func_name,
                ),
            );
        }

        Ok(result)
    }

    /// Get all attribute value ids with a boolean for each telling whether it is using a different prototype from the schema variant
    pub async fn list_attributes_with_overridden(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> AttributeValueResult<HashMap<AttributeValueId, bool>> {
        let component_av_ctx = AttributeReadContext {
            prop_id: None,
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(component_id),
        };

        let prop_av_ctx = AttributeReadContext {
            prop_id: None,
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(ComponentId::NONE),
        };

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_ATTRIBUTES_WITH_OVERRIDDEN,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &prop_av_ctx,
                    &component_av_ctx,
                    &component_id,
                ],
            )
            .await?;

        let result: HashMap<AttributeValueId, bool> = HashMap::from_iter(
            rows.iter()
                .map(|row| (row.get("attribute_value_id"), row.get("overridden"))),
        );

        Ok(result)
    }

    pub async fn remove_dependency_summaries_for_deleted_values(
        ctx: &DalContext,
    ) -> AttributeValueResult<()> {
        ctx.txns()
            .await?
            .pg()
            .execute(
                "SELECT clear_dependencies_for_deleted_values_v1($1, $2)",
                &[ctx.tenancy(), ctx.visibility()],
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
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
