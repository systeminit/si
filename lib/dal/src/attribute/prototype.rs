//! An [`AttributePrototype`] represents, for a specific attribute:
//!
//!   * Which context the following applies to ([`AttributeContext`](crate::AttributeContext))
//!   * The function that should be run to find its value.
//!   * In the case that the [`Prop`](crate::Prop) is the child of an
//!     [`Array`](crate::prop::PropKind::Array): Which index in the `Array` the value
//!     is for.
//!   * In the case that the [`Prop`](crate::Prop) is the child of a
//!     [`Map`](crate::prop::PropKind::Map): Which key of the `Map` the value is
//!     for.

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute::{
        context::{AttributeContext, AttributeContextError},
        value::{AttributeValue, AttributeValueError, AttributeValueId},
    },
    func::FuncId,
    func::{
        binding::{FuncBindingError, FuncBindingId},
        binding_return_value::{FuncBindingReturnValueError, FuncBindingReturnValueId},
    },
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_has_many,
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributeReadContext, ComponentId,
    DalContext, ExternalProviderId, HistoryEventError, InternalProviderId, PropKind,
    SchemaVariantId, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};

pub mod argument;

const ARGUMENT_VALUES_BY_NAME_FOR_HEAD_COMPONENT_ID: &str = include_str!(
    "../queries/attribute_prototype/argument_values_by_name_for_head_component_id.sql"
);
const ATTRIBUTE_VALUES_IN_CONTEXT_OR_GREATER: &str =
    include_str!("../queries/attribute_prototype/attribute_values_in_context_or_greater.sql");
const LIST_BY_HEAD_FROM_EXTERNAL_PROVIDER_USE_WITH_TAIL: &str = include_str!(
    "../queries/attribute_prototype/list_by_head_from_external_provider_use_with_tail.sql"
);
const LIST_FROM_INTERNAL_PROVIDER_USE: &str =
    include_str!("../queries/attribute_prototype/list_from_internal_provider_use.sql");
const LIST_FOR_CONTEXT: &str = include_str!("../queries/attribute_prototype/list_for_context.sql");
const FIND_WITH_PARENT_VALUE_AND_KEY_FOR_CONTEXT: &str =
    include_str!("../queries/attribute_prototype/find_with_parent_value_and_key_for_context.sql");
const FIND_FOR_FUNC: &str = include_str!("../queries/attribute_prototype/find_for_func.sql");
const FIND_FOR_CONTEXT_AND_KEY: &str =
    include_str!("../queries/attribute_prototype/find_for_context_and_key.sql");
const FIND_FOR_CONTEXT_NULL_KEY: &str =
    include_str!("../queries/attribute_prototype/find_for_context_null_key.sql");
const FIND_FOR_FUNC_AS_VARIANT_AND_COMPONENT: &str =
    include_str!("../queries/attribute_prototype/find_for_func_as_variant_and_component.sql");

#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("attribute resolver context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
    #[error("AttributePrototype is missing")]
    Missing,
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
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("attribute prototype not found: {0} ({1:?})")]
    NotFound(AttributePrototypeId, Visibility),
    #[error(
        "parent must be for an array, map, or object prop: attribute prototype id {0} is for a {1}"
    )]
    ParentNotAllowed(AttributePrototypeId, PropKind),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("cannot remove prototype with a least-specific context: {0}")]
    LeastSpecificContextPrototypeRemovalNotAllowed(AttributePrototypeId),
    #[error("cannot remove value with a least-specific context: {0}")]
    LeastSpecificContextValueRemovalNotAllowed(AttributeValueId),
    #[error("unable to construct component view for attribute function execution")]
    ComponentView,
    #[error("cannot hard delete prototype from changeset if corresponding prototype does not exist on head or if the prototype does not represent an element of a map or array")]
    HardDeletePrototypeWithNoHeadPrototypeOrKey(AttributePrototypeId),
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
    #[serde(flatten)]
    timestamp: Timestamp,

    /// The [`AttributeContext`] corresponding to the prototype.
    #[serde(flatten)]
    pub context: AttributeContext,
    /// The [`Func`](crate::Func) corresponding to the prototype.
    func_id: FuncId,
    /// An optional key used for tracking parentage.
    pub key: Option<String>,
}

/// This object is used for
/// [`AttributePrototype::list_by_head_from_external_provider_use_with_tail()`].
#[derive(Serialize, Deserialize, Debug)]
pub struct AttributePrototypeGroupByHeadComponentId {
    pub head_component_id: ComponentId,
    pub attribute_prototype: AttributePrototype,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        context: AttributeContext,
        key: Option<String>,
        parent_attribute_value_id: Option<AttributeValueId>,
    ) -> AttributePrototypeResult<Self> {
        let row = ctx.pg_txn().query_one(
            "SELECT new_attribute_prototype AS object FROM attribute_prototype_new_v1($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &func_id,
                &func_binding_id,
                &func_binding_return_value_id,
                &context,
                &key,
                &parent_attribute_value_id,
            ],
        ).await?;

        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_with_existing_value(
        ctx: &DalContext,
        func_id: FuncId,
        context: AttributeContext,
        key: Option<String>,
        parent_attribute_value_id: Option<AttributeValueId>,
        attribute_value_id: AttributeValueId,
    ) -> AttributePrototypeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT new_attribute_prototype_id AS prototype_id
                 FROM attribute_prototype_new_with_attribute_value_v1($1,
                                                                      $2,
                                                                      $3,
                                                                      $4,
                                                                      $5,
                                                                      $6,
                                                                      $7)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &context,
                    &key,
                    &parent_attribute_value_id,
                    &attribute_value_id,
                ],
            )
            .await?;
        let prototype_id: AttributePrototypeId = row.try_get("prototype_id")?;
        let object = Self::get_by_id(ctx, &prototype_id)
            .await?
            .ok_or_else(|| AttributePrototypeError::NotFound(prototype_id, *ctx.visibility()))?;

        Ok(object)
    }

    pub async fn new_with_context_only(
        ctx: &DalContext,
        func_id: FuncId,
        context: AttributeContext,
        key: Option<&str>,
    ) -> AttributePrototypeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_create_v1($1, $2, $3, $4, $5)",
                &[ctx.tenancy(), ctx.visibility(), &context, &func_id, &key],
            )
            .await?;

        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(func_id, Pk(FuncId), AttributePrototypeResult);
    standard_model_accessor!(key, Option<String>, AttributePrototypeResult);
    standard_model_has_many!(
        lookup_fn: attribute_values,
        table: "attribute_value_belongs_to_attribute_prototype",
        model_table: "attribute_values",
        returns: AttributeValue,
        result: AttributePrototypeResult,
    );

    /// Permanently deletes the [`AttributePrototype`] for the given id along with any
    /// corresponding [`AttributeValue`](crate::AttributeValue) prototype and
    /// any [`AttributePrototypeArguments`](crate::AttributePrototypeArgument)
    /// for the prototype, if and only if any of the above values are in a changeset (i.e.,
    /// not in HEAD). The effect is to revert the prototype, it's values, and arguments,
    /// to the HEAD state. Marking them as soft-deleted would propagate the deletion up to
    /// HEAD. The implementation here is almost identical to that of
    /// [`AttributePrototype::remove`](crate::AttributePrototype::remove)` but (1)
    /// checks for in_change_set and (2) hard deletes. Least-specific checks are not necessary here
    /// because we only do this for prototypes that exist only in a changeset. A corresponding
    /// prototype for this prop will exist in head, and it will take priority when this one is
    /// deleted.
    pub async fn hard_delete_if_in_changeset(
        ctx: &DalContext,
        attribute_prototype_id: &AttributePrototypeId,
    ) -> AttributePrototypeResult<()> {
        let attribute_prototype =
            match AttributePrototype::get_by_id(ctx, attribute_prototype_id).await? {
                Some(v) => v,
                None => return Ok(()),
            };

        // Ensure a prototype matching this context exists on head, or the prototype is for a
        // map/array element
        {
            let head_ctx = ctx.clone_with_head();
            let has_head_proto = AttributePrototype::find_for_context_and_key(
                &head_ctx,
                attribute_prototype.context,
                &attribute_prototype.key,
            )
            .await?
            .pop()
            .is_some();

            if !(has_head_proto || attribute_prototype.key().is_some()) {
                return Err(
                    AttributePrototypeError::HardDeletePrototypeWithNoHeadPrototypeOrKey(
                        *attribute_prototype_id,
                    ),
                );
            }
        }

        // Delete all values and arguments found for a prototype before deleting the prototype.
        let attribute_values = attribute_prototype.attribute_values(ctx).await?;
        for argument in
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *attribute_prototype_id)
                .await?
        {
            if argument.visibility().in_change_set() {
                argument.hard_delete(ctx).await?;
            }
        }
        if attribute_prototype.visibility().in_change_set() {
            standard_model::hard_unset_all_belongs_to_in_change_set(
                ctx,
                "attribute_value_belongs_to_attribute_prototype",
                attribute_prototype.id(),
            )
            .await?;
            attribute_prototype.hard_delete(ctx).await?;
        }

        // Start with the initial value(s) from the prototype and build a work queue based on the
        // value's children (and their children, recursively). Once we find the child values,
        // we can delete the current value in the queue and its prototype.
        let mut work_queue = attribute_values;
        while let Some(current_value) = work_queue.pop() {
            let child_attribute_values = current_value.child_attribute_values(ctx).await?;
            if !child_attribute_values.is_empty() {
                work_queue.extend(child_attribute_values);
            }

            // Delete the prototype if we find one and if its context is not "least-specific".
            if let Some(current_prototype) = current_value.attribute_prototype(ctx).await? {
                // Delete all arguments found for a prototype before deleting the prototype.
                for argument in AttributePrototypeArgument::list_for_attribute_prototype(
                    ctx,
                    *current_prototype.id(),
                )
                .await?
                {
                    if argument.visibility().in_change_set() {
                        argument.hard_delete(ctx).await?;
                    }
                }
                if current_prototype.visibility().in_change_set() {
                    standard_model::hard_unset_all_belongs_to_in_change_set(
                        ctx,
                        "attribute_value_belongs_to_attribute_prototype",
                        current_prototype.id(),
                    )
                    .await?;
                    current_prototype.hard_delete(ctx).await?;
                }
            }

            if current_value.visibility().in_change_set() {
                standard_model::hard_unset_belongs_to_in_change_set(
                    ctx,
                    "attribute_value_belongs_to_attribute_prototype",
                    current_value.id(),
                )
                .await?;
                standard_model::hard_unset_belongs_to_in_change_set(
                    ctx,
                    "attribute_value_belongs_to_attribute_value",
                    current_value.id(),
                )
                .await?;
                standard_model::hard_unset_all_belongs_to_in_change_set(
                    ctx,
                    "attribute_value_belongs_to_attribute_value",
                    current_value.id(),
                )
                .await?;
                current_value.hard_delete(ctx).await?;
            }
        }
        Ok(())
    }

    /// Deletes the [`AttributePrototype`] corresponding to a provided ID. Before deletion occurs,
    /// its corresponding [`AttributeValue`](crate::AttributeValue), all of its child values
    /// (and their children, recursively) and those children's prototypes are deleted. Any value or
    /// prototype that could not be found or does not exist is assumed to have already been deleted
    /// or never existed. Moreover, before deletion of the [`AttributePrototype`] occurs, we delete
    /// all [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) that belong to the
    /// prototype.
    ///
    /// Caution: this should be used rather than [`StandardModel::delete_by_id()`] when deleting an
    /// [`AttributePrototype`]. That method should never be called directly.
    pub async fn remove(
        ctx: &DalContext,
        attribute_prototype_id: &AttributePrototypeId,
    ) -> AttributePrototypeResult<()> {
        // Get the prototype for the given id. Once we get its corresponding value, we can delete
        // the prototype.
        let mut attribute_prototype =
            match AttributePrototype::get_by_id(ctx, attribute_prototype_id).await? {
                Some(v) => v,
                None => return Ok(()),
            };

        let parent_proto_is_map_or_array_element = attribute_prototype.key().is_some();
        if attribute_prototype.context.is_least_specific() && !parent_proto_is_map_or_array_element
        {
            return Err(
                AttributePrototypeError::LeastSpecificContextPrototypeRemovalNotAllowed(
                    *attribute_prototype_id,
                ),
            );
        }

        // Delete all values and arguments found for a prototype before deleting the prototype.
        let attribute_values = attribute_prototype.attribute_values(ctx).await?;
        for mut argument in
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *attribute_prototype_id)
                .await?
        {
            argument.delete_by_id(ctx).await?;
        }
        standard_model::unset_all_belongs_to(
            ctx,
            "attribute_value_belongs_to_attribute_prototype",
            attribute_prototype.id(),
        )
        .await?;
        attribute_prototype.delete_by_id(ctx).await?;

        // Start with the initial value(s) from the prototype and build a work queue based on the
        // value's children (and their children, recursively). Once we find the child values,
        // we can delete the current value in the queue and its prototype.
        let mut work_queue = attribute_values;
        while let Some(mut current_value) = work_queue.pop() {
            let child_attribute_values = current_value.child_attribute_values(ctx).await?;
            if !child_attribute_values.is_empty() {
                work_queue.extend(child_attribute_values);
            }

            // Delete the prototype if we find one and if its context is not "least-specific".
            if let Some(mut current_prototype) = current_value.attribute_prototype(ctx).await? {
                if current_prototype.context.is_least_specific()
                    && !parent_proto_is_map_or_array_element
                {
                    return Err(
                        AttributePrototypeError::LeastSpecificContextPrototypeRemovalNotAllowed(
                            *current_prototype.id(),
                        ),
                    );
                }
                // Delete all arguments found for a prototype before deleting the prototype.
                for mut argument in AttributePrototypeArgument::list_for_attribute_prototype(
                    ctx,
                    *current_prototype.id(),
                )
                .await?
                {
                    argument.delete_by_id(ctx).await?;
                }
                standard_model::unset_all_belongs_to(
                    ctx,
                    "attribute_value_belongs_to_attribute_prototype",
                    current_prototype.id(),
                )
                .await?;
                current_prototype.delete_by_id(ctx).await?;
            }

            // Delete the value if its context is not "least-specific".
            if current_value.context.is_least_specific() && !parent_proto_is_map_or_array_element {
                return Err(
                    AttributePrototypeError::LeastSpecificContextValueRemovalNotAllowed(
                        *current_value.id(),
                    ),
                );
            }
            current_value.unset_attribute_prototype(ctx).await?;
            current_value.unset_parent_attribute_value(ctx).await?;
            standard_model::unset_all_belongs_to(
                ctx,
                "attribute_value_belongs_to_attribute_value",
                current_value.id(),
            )
            .await?;
            current_value.delete_by_id(ctx).await?;
        }
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_for_context(
        ctx: &DalContext,
        context: AttributeContext,
    ) -> AttributePrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context,
                    &context.prop_id(),
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }

    #[tracing::instrument(skip_all)]
    pub async fn find_with_parent_value_and_key_for_context(
        ctx: &DalContext,
        parent_attribute_value_id: Option<AttributeValueId>,
        key: Option<String>,
        context: AttributeContext,
    ) -> AttributePrototypeResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_WITH_PARENT_VALUE_AND_KEY_FOR_CONTEXT,
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

    /// List [`Vec<Self>`] that depend on a provided [`InternalProviderId`](crate::InternalProvider).
    pub async fn list_from_internal_provider_use(
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(
                LIST_FROM_INTERNAL_PROVIDER_USE,
                &[ctx.tenancy(), ctx.visibility(), &internal_provider_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// List [`Vec<Self>`] that depend on a provided [`ExternalProviderId`](crate::ExternalProvider)
    /// and _tail_ [`ComponentId`](crate::Component).
    pub async fn list_by_head_from_external_provider_use_with_tail(
        ctx: &DalContext,
        external_provider_id: ExternalProviderId,
        tail_component_id: ComponentId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeGroupByHeadComponentId>> {
        let rows = ctx
            .pg_txn()
            .query(
                LIST_BY_HEAD_FROM_EXTERNAL_PROVIDER_USE_WITH_TAIL,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &external_provider_id,
                    &tail_component_id,
                ],
            )
            .await?;

        let mut result = Vec::new();
        for row in rows.into_iter() {
            let head_component_id: ComponentId = row.try_get("head_component_id")?;

            let attribute_prototype_json: serde_json::Value = row.try_get("object")?;
            let attribute_prototype = serde_json::from_value(attribute_prototype_json)?;

            result.push(AttributePrototypeGroupByHeadComponentId {
                head_component_id,
                attribute_prototype,
            });
        }
        Ok(result)
    }

    pub async fn argument_values(
        &self,
        ctx: &DalContext,
        attribute_write_context: AttributeContext,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeArgumentValues>> {
        let rows = ctx
            .pg_txn()
            .query(
                ARGUMENT_VALUES_BY_NAME_FOR_HEAD_COMPONENT_ID,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &self.id,
                    &attribute_write_context.component_id(),
                    &attribute_write_context,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// List [`AttributeValues`](crate::AttributeValue) that belong to a provided [`AttributePrototypeId`](Self)
    /// and whose context contains the provided [`AttributeReadContext`](crate::AttributeReadContext)
    /// or are "more-specific" than the provided [`AttributeReadContext`](crate::AttributeReadContext).
    pub async fn attribute_values_in_context_or_greater(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        context: AttributeReadContext,
    ) -> AttributePrototypeResult<Vec<AttributeValue>> {
        let rows = ctx
            .pg_txn()
            .query(
                ATTRIBUTE_VALUES_IN_CONTEXT_OR_GREATER,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &context,
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    #[async_recursion]
    async fn create_intermediate_proxy_values(
        ctx: &DalContext,
        parent_attribute_value_id: Option<AttributeValueId>,
        prototype_id: AttributePrototypeId,
        context: AttributeContext,
    ) -> AttributePrototypeResult<()> {
        if context.is_least_specific() {
            return Ok(());
        }

        if (AttributeValue::find_with_parent_and_prototype_for_context(
            ctx,
            parent_attribute_value_id,
            prototype_id,
            context,
        )
        .await?)
            .is_none()
        {
            // Need to create a proxy to the next lowest level
            Self::create_intermediate_proxy_values(
                ctx,
                parent_attribute_value_id,
                prototype_id,
                context.less_specific()?,
            )
            .await?;

            if let Some(proxy_target) = AttributeValue::find_with_parent_and_prototype_for_context(
                ctx,
                parent_attribute_value_id,
                prototype_id,
                context.less_specific()?,
            )
            .await?
            {
                // Create the proxy at this level
                let mut proxy_attribute_value = AttributeValue::new(
                    ctx,
                    proxy_target.func_binding_id(),
                    proxy_target.func_binding_return_value_id(),
                    context,
                    proxy_target.key().map(|k| k.to_string()),
                )
                .await?;
                proxy_attribute_value
                    .set_proxy_for_attribute_value_id(ctx, Some(*proxy_target.id()))
                    .await?;
                proxy_attribute_value
                    .set_attribute_prototype(ctx, &prototype_id)
                    .await?
            } else {
                return Err(AttributePrototypeError::MissingValue(
                    *ctx.tenancy(),
                    *ctx.visibility(),
                    prototype_id,
                    parent_attribute_value_id,
                ));
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_for_context(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        context: AttributeContext,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        parent_attribute_value_id: Option<AttributeValueId>,
        existing_attribute_value_id: Option<AttributeValueId>,
    ) -> AttributePrototypeResult<AttributePrototypeId> {
        let given_attribute_prototype = Self::get_by_id(ctx, &attribute_prototype_id)
            .await?
            .ok_or_else(|| {
                AttributePrototypeError::NotFound(attribute_prototype_id, *ctx.visibility())
            })?;

        // If the AttributePrototype we were given isn't for the _specific_ context that we're
        // trying to update, make a new one. This is necessary so that we don't end up changing the
        // prototype for a context less specific than the one that we're trying to update.
        let mut attribute_prototype = if given_attribute_prototype.context == context {
            given_attribute_prototype
        } else if let Some(attribute_value_id) = existing_attribute_value_id {
            // Create new prototype with an existing value and clone the arguments of the given prototype into the new one.
            let prototype = Self::new_with_existing_value(
                ctx,
                func_id,
                context,
                given_attribute_prototype.key().map(|k| k.to_string()),
                parent_attribute_value_id,
                attribute_value_id,
            )
            .await?;

            let mut value = AttributeValue::get_by_id(ctx, &attribute_value_id)
                .await?
                .ok_or_else(|| {
                    AttributePrototypeError::MissingValue(
                        *ctx.tenancy(),
                        *ctx.visibility(),
                        *prototype.id(),
                        Some(attribute_value_id),
                    )
                })?;
            value.set_func_binding_id(ctx, func_binding_id).await?;

            prototype
        } else {
            // Create new prototype and clone the arguments of the given prototype into the new one.
            Self::new(
                ctx,
                func_id,
                func_binding_id,
                func_binding_return_value_id,
                context,
                given_attribute_prototype.key().map(|k| k.to_string()),
                parent_attribute_value_id,
            )
            .await?
        };

        attribute_prototype.set_func_id(ctx, func_id).await?;

        Ok(*attribute_prototype.id())
    }

    pub async fn find_for_func(
        ctx: &DalContext,
        func_id: &FuncId,
    ) -> AttributePrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), func_id])
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_func_as_variant_and_component(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> AttributePrototypeResult<Vec<(SchemaVariantId, ComponentId)>> {
        let mut result = vec![];

        let rows = ctx
            .pg_txn()
            .query(
                FIND_FOR_FUNC_AS_VARIANT_AND_COMPONENT,
                &[ctx.tenancy(), ctx.visibility(), &func_id],
            )
            .await?;

        for row in rows.into_iter() {
            let schema_variant_id: SchemaVariantId = row.try_get("schema_variant_id")?;
            let component_id: ComponentId = row.try_get("component_id")?;

            result.push((schema_variant_id, component_id));
        }

        Ok(result)
    }

    pub async fn find_for_context_and_key(
        ctx: &DalContext,
        context: AttributeContext,
        key: &Option<String>,
    ) -> AttributePrototypeResult<Vec<Self>> {
        let rows = if key.is_some() {
            ctx.txns()
                .pg()
                .query(
                    FIND_FOR_CONTEXT_AND_KEY,
                    &[
                        ctx.tenancy(),
                        ctx.visibility(),
                        &context.prop_id(),
                        &context.internal_provider_id(),
                        &context.external_provider_id(),
                        &context.component_id(),
                        &key,
                    ],
                )
                .await?
        } else {
            ctx.txns()
                .pg()
                .query(
                    FIND_FOR_CONTEXT_NULL_KEY,
                    &[
                        ctx.tenancy(),
                        ctx.visibility(),
                        &context.prop_id(),
                        &context.internal_provider_id(),
                        &context.external_provider_id(),
                        &context.component_id(),
                    ],
                )
                .await?
        };

        Ok(standard_model::objects_from_rows(rows)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributePrototypeArgumentValues {
    pub attribute_prototype_id: AttributePrototypeId,
    pub argument_name: String,
    pub values: Vec<serde_json::Value>,
}
