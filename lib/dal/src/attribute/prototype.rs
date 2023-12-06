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

use content_store::{ContentHash, Store};
use petgraph::prelude::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::NodeIndex;
use crate::workspace_snapshot::node_weight::{
    NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, DalContext, FuncId, InternalProviderId, PropId, StandardModel, Timestamp, TransactionsError,
};

pub mod argument;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("attribute prototype {0} is missing a function edge")]
    MissingFunction(AttributePrototypeId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributePrototypeResult<T> = Result<T, AttributePrototypeError>;

pk!(AttributePrototypeId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototype {
    id: AttributePrototypeId,
    timestamp: Timestamp,
}

#[derive(Debug, PartialEq)]
pub struct AttributePrototypeGraphNode {
    id: AttributePrototypeId,
    content_address: ContentAddress,
    content: AttributePrototypeContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum AttributePrototypeContent {
    V1(AttributePrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributePrototypeContentV1 {
    pub timestamp: Timestamp,
}

impl AttributePrototypeGraphNode {
    pub fn assemble(
        id: impl Into<AttributePrototypeId>,
        content_hash: ContentHash,
        content: AttributePrototypeContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::AttributePrototype(content_hash),
            content,
        }
    }
}

impl AttributePrototype {
    pub fn assemble(id: AttributePrototypeId, inner: &AttributePrototypeContentV1) -> Self {
        let inner: AttributePrototypeContentV1 = inner.to_owned();
        Self {
            id,
            timestamp: inner.timestamp,
        }
    }

    pub fn id(&self) -> AttributePrototypeId {
        self.id
    }

    // NOTE(nick): all incoming edges to an attribute prototype must come from one of two places:
    //   - an attribute value whose lineage comes from a component
    //   - a prop or provider whose lineage comes from a schema variant
    // Outgoing edges from an attribute prototype are used for intra and inter component relationships.
    pub fn new(ctx: &DalContext, func_id: FuncId) -> AttributePrototypeResult<Self> {
        let timestamp = Timestamp::now();

        let content = AttributePrototypeContentV1 { timestamp };
        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&AttributePrototypeContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributePrototype(hash))?;
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let _node_index = workspace_snapshot.add_node(node_weight)?;

        workspace_snapshot.add_edge(
            id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_id.into(),
        )?;

        Ok(AttributePrototype::assemble(
            AttributePrototypeId::from(id),
            &content,
        ))
    }

    pub fn func_id(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<FuncId> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(prototype_id, EdgeWeightKindDiscriminants::Use)?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index)?;
            if NodeWeightDiscriminants::Func == node_weight.into() {
                return Ok(node_weight.id().into());
            }
        }

        Err(AttributePrototypeError::MissingFunction(prototype_id))
    }

    pub fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
        key: &Option<String>,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(prop_id, Direction::Outgoing)?
            .find(|edge_ref| {
                if let EdgeWeightKind::Prototype(maybe_key) = edge_ref.weight().kind() {
                    maybe_key == key
                } else {
                    false
                }
            })
            .map(|edge_ref| edge_ref.target())
        {
            let node_weight = workspace_snapshot.get_node_weight(prototype_idx)?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub fn update_func_by_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_id: FuncId,
    ) -> AttributePrototypeResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let attribute_prototype_idx =
            workspace_snapshot.get_node_index_by_id(attribute_prototype_id)?;

        let current_func_node_idx = workspace_snapshot
            .edges_directed(attribute_prototype_id, Direction::Outgoing)?
            .find(|edge_ref| edge_ref.weight().kind() == &EdgeWeightKind::Use)
            .map(|edge_ref| edge_ref.target())
            .ok_or(AttributePrototypeError::MissingFunction(
                attribute_prototype_id,
            ))?;

        let change_set = ctx.change_set_pointer()?;
        workspace_snapshot.remove_edge(
            change_set,
            attribute_prototype_idx,
            current_func_node_idx,
            EdgeWeightKindDiscriminants::Use,
        )?;

        workspace_snapshot.add_edge(
            attribute_prototype_id.into(),
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_id.into(),
        )?;

        Ok(())
    }

    pub fn remove(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        workspace_snapshot.remove_incoming_edges_of_kind(
            ctx.change_set_pointer()?,
            prototype_id,
            EdgeWeightKindDiscriminants::Prototype,
        )?;

        Ok(())
    }
}

// /// This object is used for
// /// [`AttributePrototype::list_by_head_from_external_provider_use_with_tail()`].
// #[derive(Serialize, Deserialize, Debug)]
// pub struct AttributePrototypeGroupByHeadComponentId {
//     pub head_component_id: ComponentId,
//     pub attribute_prototype: AttributePrototype,
// }

// impl AttributePrototype {
//     #[allow(clippy::too_many_arguments)]
//     #[instrument(skip_all)]
//     pub async fn new_with_existing_value(
//         ctx: &DalContext,
//         func_id: FuncId,
//         context: AttributeContext,
//         key: Option<String>,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         attribute_value_id: AttributeValueId,
//     ) -> AttributePrototypeResult<Self> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT new_attribute_prototype_id AS prototype_id
//                  FROM attribute_prototype_new_with_attribute_value_v1($1,
//                                                                       $2,
//                                                                       $3,
//                                                                       $4,
//                                                                       $5,
//                                                                       $6,
//                                                                       $7)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &func_id,
//                     &context,
//                     &key,
//                     &parent_attribute_value_id,
//                     &attribute_value_id,
//                 ],
//             )
//             .await?;
//         let prototype_id: AttributePrototypeId = row.try_get("prototype_id")?;
//         let object = Self::get_by_id(ctx, &prototype_id)
//             .await?
//             .ok_or_else(|| AttributePrototypeError::NotFound(prototype_id, *ctx.visibility()))?;

//         Ok(object)
//     }

//     pub async fn new_with_context_only(
//         ctx: &DalContext,
//         func_id: FuncId,
//         context: AttributeContext,
//         key: Option<&str>,
//     ) -> AttributePrototypeResult<Self> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM attribute_prototype_create_v1($1, $2, $3, $4, $5)",
//                 &[ctx.tenancy(), ctx.visibility(), &context, &func_id, &key],
//             )
//             .await?;

//         Ok(standard_model::finish_create_from_row(ctx, row).await?)
//     }

//     standard_model_accessor!(func_id, Pk(FuncId), AttributePrototypeResult);
//     standard_model_accessor!(key, Option<String>, AttributePrototypeResult);
//     standard_model_has_many!(
//         lookup_fn: attribute_values,
//         table: "attribute_value_belongs_to_attribute_prototype",
//         model_table: "attribute_values",
//         returns: AttributeValue,
//         result: AttributePrototypeResult,
//     );

//     /// Permanently deletes the [`AttributePrototype`] for the given id along with any
//     /// corresponding [`AttributeValue`](crate::AttributeValue) prototype and
//     /// any [`AttributePrototypeArguments`](crate::AttributePrototypeArgument)
//     /// for the prototype, if and only if any of the above values are in a changeset (i.e.,
//     /// not in HEAD). The effect is to revert the prototype, it's values, and arguments,
//     /// to the HEAD state. Marking them as soft-deleted would propagate the deletion up to
//     /// HEAD. The implementation here is almost identical to that of
//     /// [`AttributePrototype::remove`](crate::AttributePrototype::remove)` but (1)
//     /// checks for in_change_set and (2) hard deletes. Least-specific checks are not necessary here
//     /// because we only do this for prototypes that exist only in a changeset. A corresponding
//     /// prototype for this prop will exist in head, and it will take priority when this one is
//     /// deleted.
//     pub async fn hard_delete_if_in_changeset(
//         ctx: &DalContext,
//         attribute_prototype_id: &AttributePrototypeId,
//     ) -> AttributePrototypeResult<()> {
//         let attribute_prototype =
//             match AttributePrototype::get_by_id(ctx, attribute_prototype_id).await? {
//                 Some(v) => v,
//                 None => return Ok(()),
//             };

//         // Ensure a prototype matching this context exists on head, or the prototype is for a
//         // map/array element
//         {
//             let head_ctx = ctx.clone_with_head();
//             let has_head_proto = AttributePrototype::find_for_context_and_key(
//                 &head_ctx,
//                 attribute_prototype.context,
//                 &attribute_prototype.key,
//             )
//             .await?
//             .pop()
//             .is_some();

//             if !(has_head_proto || attribute_prototype.key().is_some()) {
//                 return Err(
//                     AttributePrototypeError::HardDeletePrototypeWithNoHeadPrototypeOrKey(
//                         *attribute_prototype_id,
//                     ),
//                 );
//             }
//         }

//         // Delete all values and arguments found for a prototype before deleting the prototype.
//         let attribute_values = attribute_prototype.attribute_values(ctx).await?;
//         for argument in
//             AttributePrototypeArgument::list_for_attribute_prototype(ctx, *attribute_prototype_id)
//                 .await?
//         {
//             if argument.visibility().in_change_set() {
//                 argument.hard_delete(ctx).await?;
//             }
//         }
//         if attribute_prototype.visibility().in_change_set() {
//             standard_model::hard_unset_all_belongs_to_in_change_set(
//                 ctx,
//                 "attribute_value_belongs_to_attribute_prototype",
//                 attribute_prototype.id(),
//             )
//             .await?;
//             attribute_prototype.hard_delete(ctx).await?;
//         }

//         // Start with the initial value(s) from the prototype and build a work queue based on the
//         // value's children (and their children, recursively). Once we find the child values,
//         // we can delete the current value in the queue and its prototype.
//         let mut work_queue = attribute_values;
//         while let Some(current_value) = work_queue.pop() {
//             let child_attribute_values = current_value.child_attribute_values(ctx).await?;
//             if !child_attribute_values.is_empty() {
//                 work_queue.extend(child_attribute_values);
//             }

//             // Delete the prototype if we find one and if its context is not "least-specific".
//             if let Some(current_prototype) = current_value.attribute_prototype(ctx).await? {
//                 // Delete all arguments found for a prototype before deleting the prototype.
//                 for argument in AttributePrototypeArgument::list_for_attribute_prototype(
//                     ctx,
//                     *current_prototype.id(),
//                 )
//                 .await?
//                 {
//                     if argument.visibility().in_change_set() {
//                         argument.hard_delete(ctx).await?;
//                     }
//                 }
//                 if current_prototype.visibility().in_change_set() {
//                     standard_model::hard_unset_all_belongs_to_in_change_set(
//                         ctx,
//                         "attribute_value_belongs_to_attribute_prototype",
//                         current_prototype.id(),
//                     )
//                     .await?;
//                     current_prototype.hard_delete(ctx).await?;
//                 }
//             }

//             if current_value.visibility().in_change_set() {
//                 standard_model::hard_unset_belongs_to_in_change_set(
//                     ctx,
//                     "attribute_value_belongs_to_attribute_prototype",
//                     current_value.id(),
//                 )
//                 .await?;
//                 standard_model::hard_unset_belongs_to_in_change_set(
//                     ctx,
//                     "attribute_value_belongs_to_attribute_value",
//                     current_value.id(),
//                 )
//                 .await?;
//                 standard_model::hard_unset_all_belongs_to_in_change_set(
//                     ctx,
//                     "attribute_value_belongs_to_attribute_value",
//                     current_value.id(),
//                 )
//                 .await?;
//                 current_value.hard_delete(ctx).await?;
//             }
//         }
//         Ok(())
//     }

//     /// Deletes the [`AttributePrototype`] corresponding to a provided ID. Before deletion occurs,
//     /// its corresponding [`AttributeValue`](crate::AttributeValue), all of its child values
//     /// (and their children, recursively) and those children's prototypes are deleted. Any value or
//     /// prototype that could not be found or does not exist is assumed to have already been deleted
//     /// or never existed. Moreover, before deletion of the [`AttributePrototype`] occurs, we delete
//     /// all [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) that belong to the
//     /// prototype.
//     ///
//     /// Caution: this should be used rather than [`StandardModel::delete_by_id()`] when deleting an
//     /// [`AttributePrototype`]. That method should never be called directly.
//     ///
//     /// Normally we forbid deleting "least specific" attribute prototypes, that is, prototypes
//     /// at the schema variant level, but we need to do so when removing a schema variant and
//     /// all its associated objects. To make this possible, set `force` to `true`
//     pub async fn remove(
//         ctx: &DalContext,
//         attribute_prototype_id: &AttributePrototypeId,
//         force: bool,
//     ) -> AttributePrototypeResult<()> {
//         // Get the prototype for the given id. Once we get its corresponding value, we can delete
//         // the prototype.
//         let mut attribute_prototype =
//             match AttributePrototype::get_by_id(ctx, attribute_prototype_id).await? {
//                 Some(v) => v,
//                 None => return Ok(()),
//             };

//         let parent_proto_is_map_or_array_element = attribute_prototype.key().is_some();
//         if attribute_prototype.context.is_least_specific()
//             && !parent_proto_is_map_or_array_element
//             && !force
//         {
//             return Err(
//                 AttributePrototypeError::LeastSpecificContextPrototypeRemovalNotAllowed(
//                     *attribute_prototype_id,
//                 ),
//             );
//         }

//         // Delete all values and arguments found for a prototype before deleting the prototype.
//         let attribute_values = attribute_prototype.attribute_values(ctx).await?;
//         for mut argument in
//             AttributePrototypeArgument::list_for_attribute_prototype(ctx, *attribute_prototype_id)
//                 .await?
//         {
//             argument.delete_by_id(ctx).await?;
//         }
//         standard_model::unset_all_belongs_to(
//             ctx,
//             "attribute_value_belongs_to_attribute_prototype",
//             attribute_prototype.id(),
//         )
//         .await?;
//         attribute_prototype.delete_by_id(ctx).await?;

//         // Start with the initial value(s) from the prototype and build a work queue based on the
//         // value's children (and their children, recursively). Once we find the child values,
//         // we can delete the current value in the queue and its prototype.
//         let mut work_queue = attribute_values;
//         while let Some(mut current_value) = work_queue.pop() {
//             let child_attribute_values = current_value.child_attribute_values(ctx).await?;
//             if !child_attribute_values.is_empty() {
//                 work_queue.extend(child_attribute_values);
//             }

//             // Delete the prototype if we find one and if its context is not "least-specific".
//             if let Some(mut current_prototype) = current_value.attribute_prototype(ctx).await? {
//                 if current_prototype.context.is_least_specific()
//                     && !parent_proto_is_map_or_array_element
//                     && !force
//                 {
//                     return Err(
//                         AttributePrototypeError::LeastSpecificContextPrototypeRemovalNotAllowed(
//                             *current_prototype.id(),
//                         ),
//                     );
//                 }
//                 // Delete all arguments found for a prototype before deleting the prototype.
//                 for mut argument in AttributePrototypeArgument::list_for_attribute_prototype(
//                     ctx,
//                     *current_prototype.id(),
//                 )
//                 .await?
//                 {
//                     argument.delete_by_id(ctx).await?;
//                 }
//                 standard_model::unset_all_belongs_to(
//                     ctx,
//                     "attribute_value_belongs_to_attribute_prototype",
//                     current_prototype.id(),
//                 )
//                 .await?;
//                 current_prototype.delete_by_id(ctx).await?;
//             }

//             // Delete the value if its context is not "least-specific".
//             if current_value.context.is_least_specific()
//                 && !parent_proto_is_map_or_array_element
//                 && !force
//             {
//                 return Err(
//                     AttributePrototypeError::LeastSpecificContextValueRemovalNotAllowed(
//                         *current_value.id(),
//                     ),
//                 );
//             }
//             current_value.unset_attribute_prototype(ctx).await?;
//             current_value.unset_parent_attribute_value(ctx).await?;
//             standard_model::unset_all_belongs_to(
//                 ctx,
//                 "attribute_value_belongs_to_attribute_value",
//                 current_value.id(),
//             )
//             .await?;
//             current_value.delete_by_id(ctx).await?;
//         }
//         Ok(())
//     }

//     #[instrument(skip_all)]
//     pub async fn list_prototype_funcs_by_context_and_backend_response_type(
//         ctx: &DalContext,
//         context: AttributeContext,
//         backend_response_type: FuncBackendResponseType,
//     ) -> AttributePrototypeResult<Vec<(Self, Func)>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FUNCS_FOR_CONTEXT_AND_BACKEND_RESPONSE_TYPE,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &context.prop_id(),
//                     &backend_response_type.as_ref(),
//                 ],
//             )
//             .await?;

//         let mut result = Vec::new();
//         for row in rows.into_iter() {
//             let func_json: serde_json::Value = row.try_get("func_object")?;
//             let func: Func = serde_json::from_value(func_json)?;

//             let ap_json: serde_json::Value = row.try_get("prototype_object")?;
//             let ap: Self = serde_json::from_value(ap_json)?;

//             result.push((ap, func));
//         }

//         Ok(result)
//     }

//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> AttributePrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     #[instrument(skip_all)]
//     pub async fn list_for_context(
//         ctx: &DalContext,
//         context: AttributeContext,
//     ) -> AttributePrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &context.prop_id(),
//                 ],
//             )
//             .await?;
//         let object = standard_model::objects_from_rows(rows)?;
//         Ok(object)
//     }

//     #[tracing::instrument(skip_all)]
//     pub async fn find_with_parent_value_and_key_for_context(
//         ctx: &DalContext,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         key: Option<String>,
//         context: AttributeContext,
//     ) -> AttributePrototypeResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_WITH_PARENT_VALUE_AND_KEY_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &parent_attribute_value_id,
//                     &key,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::option_object_from_row(row)?)
//     }

//     /// List [`Vec<Self>`] that depend on a provided [`InternalProviderId`](crate::InternalProvider).
//     pub async fn list_from_internal_provider_use(
//         ctx: &DalContext,
//         internal_provider_id: InternalProviderId,
//     ) -> AttributePrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FROM_INTERNAL_PROVIDER_USE,
//                 &[ctx.tenancy(), ctx.visibility(), &internal_provider_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// List [`Vec<Self>`] that depend on a provided [`ExternalProviderId`](crate::ExternalProvider)
//     /// and _tail_ [`ComponentId`](crate::Component).
//     pub async fn list_by_head_from_external_provider_use_with_tail(
//         ctx: &DalContext,
//         external_provider_id: ExternalProviderId,
//         tail_component_id: ComponentId,
//     ) -> AttributePrototypeResult<Vec<AttributePrototypeGroupByHeadComponentId>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_BY_HEAD_FROM_EXTERNAL_PROVIDER_USE_WITH_TAIL,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &external_provider_id,
//                     &tail_component_id,
//                 ],
//             )
//             .await?;

//         let mut result = Vec::new();
//         for row in rows.into_iter() {
//             let head_component_id: ComponentId = row.try_get("head_component_id")?;

//             let attribute_prototype_json: serde_json::Value = row.try_get("object")?;
//             let attribute_prototype = serde_json::from_value(attribute_prototype_json)?;

//             result.push(AttributePrototypeGroupByHeadComponentId {
//                 head_component_id,
//                 attribute_prototype,
//             });
//         }
//         Ok(result)
//     }

//     pub async fn argument_values(
//         &self,
//         ctx: &DalContext,
//         attribute_write_context: AttributeContext,
//     ) -> AttributePrototypeResult<Vec<AttributePrototypeArgumentValues>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 ARGUMENT_VALUES_BY_NAME_FOR_HEAD_COMPONENT_ID,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &self.id,
//                     &attribute_write_context.component_id(),
//                     &attribute_write_context,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// List [`AttributeValues`](crate::AttributeValue) that belong to a provided [`AttributePrototypeId`](Self)
//     /// and whose context contains the provided [`AttributeReadContext`](crate::AttributeReadContext)
//     /// or are "more-specific" than the provided [`AttributeReadContext`](crate::AttributeReadContext).
//     pub async fn attribute_values_in_context_or_greater(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         context: AttributeReadContext,
//     ) -> AttributePrototypeResult<Vec<AttributeValue>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 ATTRIBUTE_VALUES_IN_CONTEXT_OR_GREATER,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &context,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     #[instrument(skip_all)]
//     #[allow(clippy::too_many_arguments)]
//     #[async_recursion]
//     async fn create_intermediate_proxy_values(
//         ctx: &DalContext,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         prototype_id: AttributePrototypeId,
//         context: AttributeContext,
//     ) -> AttributePrototypeResult<()> {
//         if context.is_least_specific() {
//             return Ok(());
//         }

//         if (AttributeValue::find_with_parent_and_prototype_for_context(
//             ctx,
//             parent_attribute_value_id,
//             prototype_id,
//             context,
//         )
//         .await?)
//             .is_none()
//         {
//             // Need to create a proxy to the next lowest level
//             Self::create_intermediate_proxy_values(
//                 ctx,
//                 parent_attribute_value_id,
//                 prototype_id,
//                 context.less_specific()?,
//             )
//             .await?;

//             if let Some(proxy_target) = AttributeValue::find_with_parent_and_prototype_for_context(
//                 ctx,
//                 parent_attribute_value_id,
//                 prototype_id,
//                 context.less_specific()?,
//             )
//             .await?
//             {
//                 // Create the proxy at this level
//                 let mut proxy_attribute_value = AttributeValue::new(
//                     ctx,
//                     proxy_target.func_binding_id(),
//                     proxy_target.func_binding_return_value_id(),
//                     context,
//                     proxy_target.key().map(|k| k.to_string()),
//                 )
//                 .await?;
//                 proxy_attribute_value
//                     .set_proxy_for_attribute_value_id(ctx, Some(*proxy_target.id()))
//                     .await?;
//                 proxy_attribute_value
//                     .set_attribute_prototype(ctx, &prototype_id)
//                     .await?
//             } else {
//                 return Err(AttributePrototypeError::MissingValue(
//                     *ctx.tenancy(),
//                     *ctx.visibility(),
//                     prototype_id,
//                     parent_attribute_value_id,
//                 ));
//             }
//         }

//         Ok(())
//     }

//     #[allow(clippy::too_many_arguments)]
//     pub async fn update_for_context(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         context: AttributeContext,
//         func_id: FuncId,
//         func_binding_id: FuncBindingId,
//         func_binding_return_value_id: FuncBindingReturnValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         existing_attribute_value_id: Option<AttributeValueId>,
//     ) -> AttributePrototypeResult<AttributePrototypeId> {
//         let given_attribute_prototype = Self::get_by_id(ctx, &attribute_prototype_id)
//             .await?
//             .ok_or_else(|| {
//                 AttributePrototypeError::NotFound(attribute_prototype_id, *ctx.visibility())
//             })?;

//         // If the AttributePrototype we were given isn't for the _specific_ context that we're
//         // trying to update, make a new one. This is necessary so that we don't end up changing the
//         // prototype for a context less specific than the one that we're trying to update.
//         let mut attribute_prototype = if given_attribute_prototype.context == context {
//             given_attribute_prototype
//         } else if let Some(attribute_value_id) = existing_attribute_value_id {
//             // Create new prototype with an existing value and clone the arguments of the given prototype into the new one.
//             let prototype = Self::new_with_existing_value(
//                 ctx,
//                 func_id,
//                 context,
//                 given_attribute_prototype.key().map(|k| k.to_string()),
//                 parent_attribute_value_id,
//                 attribute_value_id,
//             )
//             .await?;

//             let mut value = AttributeValue::get_by_id(ctx, &attribute_value_id)
//                 .await?
//                 .ok_or_else(|| {
//                     AttributePrototypeError::MissingValue(
//                         *ctx.tenancy(),
//                         *ctx.visibility(),
//                         *prototype.id(),
//                         Some(attribute_value_id),
//                     )
//                 })?;
//             value.set_func_binding_id(ctx, func_binding_id).await?;

//             prototype
//         } else {
//             // Create new prototype and clone the arguments of the given prototype into the new one.
//             Self::new(
//                 ctx,
//                 func_id,
//                 func_binding_id,
//                 func_binding_return_value_id,
//                 context,
//                 given_attribute_prototype.key().map(|k| k.to_string()),
//                 parent_attribute_value_id,
//             )
//             .await?
//         };

//         attribute_prototype.set_func_id(ctx, func_id).await?;

//         Ok(*attribute_prototype.id())
//     }

//     pub async fn find_for_func(
//         ctx: &DalContext,
//         func_id: &FuncId,
//     ) -> AttributePrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), func_id])
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_for_func_as_variant_and_component(
//         ctx: &DalContext,
//         func_id: FuncId,
//     ) -> AttributePrototypeResult<Vec<(SchemaVariantId, ComponentId)>> {
//         let mut result = vec![];

//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_FUNC_AS_VARIANT_AND_COMPONENT,
//                 &[ctx.tenancy(), ctx.visibility(), &func_id],
//             )
//             .await?;

//         for row in rows.into_iter() {
//             let schema_variant_id: SchemaVariantId = row.try_get("schema_variant_id")?;
//             let component_id: ComponentId = row.try_get("component_id")?;

//             result.push((schema_variant_id, component_id));
//         }

//         Ok(result)
//     }

//     pub async fn find_for_context_and_key(
//         ctx: &DalContext,
//         context: AttributeContext,
//         key: &Option<String>,
//     ) -> AttributePrototypeResult<Vec<Self>> {
//         let rows = if key.is_some() {
//             ctx.txns()
//                 .await?
//                 .pg()
//                 .query(
//                     FIND_FOR_CONTEXT_AND_KEY,
//                     &[
//                         ctx.tenancy(),
//                         ctx.visibility(),
//                         &context.prop_id(),
//                         &context.internal_provider_id(),
//                         &context.external_provider_id(),
//                         &context.component_id(),
//                         &key,
//                     ],
//                 )
//                 .await?
//         } else {
//             ctx.txns()
//                 .await?
//                 .pg()
//                 .query(
//                     FIND_FOR_CONTEXT_NULL_KEY,
//                     &[
//                         ctx.tenancy(),
//                         ctx.visibility(),
//                         &context.prop_id(),
//                         &context.internal_provider_id(),
//                         &context.external_provider_id(),
//                         &context.component_id(),
//                     ],
//                 )
//                 .await?
//         };

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn external_provider(
//         &self,
//         ctx: &DalContext,
//     ) -> AttributePrototypeResult<ExternalProvider> {
//         ExternalProvider::get_by_id(ctx, &self.context.external_provider_id())
//             .await?
//             .ok_or(AttributePrototypeError::ExternalProviderNotFound(
//                 self.context.external_provider_id(),
//             ))
//     }

//     pub async fn internal_provider(
//         &self,
//         ctx: &DalContext,
//     ) -> AttributePrototypeResult<InternalProvider> {
//         InternalProvider::get_by_id(ctx, &self.context.internal_provider_id())
//             .await?
//             .ok_or(AttributePrototypeError::InternalProviderNotFound(
//                 self.context.internal_provider_id(),
//             ))
//     }

//     pub async fn prop(&self, ctx: &DalContext) -> AttributePrototypeResult<Prop> {
//         Prop::get_by_id(ctx, &self.context.prop_id()).await?.ok_or(
//             AttributePrototypeError::PropNotFound(self.context.prop_id()),
//         )
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AttributePrototypeArgumentValues {
//     pub attribute_prototype_id: AttributePrototypeId,
//     pub argument_name: String,
//     pub values: Vec<serde_json::Value>,
// }
