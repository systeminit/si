use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;

pub mod batch;
pub mod prototype;
pub mod runner;

use crate::change_set::ChangeSetError;
use crate::layer_db_types::ComponentContent;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to,
    layer_db_types::{ActionPrototypeContent, DeprecatedActionContent, DeprecatedActionContentV1},
    pk, ActionKind, ActionPrototype, ActionPrototypeError, ActionPrototypeId, ChangeSetId,
    Component, ComponentError, ComponentId, DalContext, DeprecatedActionBatchError, HelperError,
    HistoryActor, HistoryEventError, Timestamp, TransactionsError, UserPk, WsEvent, WsEventError,
    WsEventResult, WsPayload,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DeprecatedActionError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    ChangeSet(#[from] ChangeSetError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("component not found for: {0}")]
    ComponentNotFoundFor(ActionId),
    #[error("action error: {0}")]
    DeprecatedActionBatch(#[from] DeprecatedActionBatchError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("history event: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("in head")]
    InHead,
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("action not found: {0}")]
    NotFound(ActionId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("action prototype not found for: {0}")]
    PrototypeNotFoundFor(ActionId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type DeprecatedActionResult<T> = Result<T, DeprecatedActionError>;

pk!(ActionId);

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct DeprecatedActionBag {
    pub component_id: ComponentId,
    pub action: DeprecatedAction,
    pub kind: ActionKind,
    pub parents: Vec<ActionId>,
}

// An Action joins an `ActionPrototype` to a `ComponentId` in a `ChangeSetId`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedAction {
    pub id: ActionId,
    pub creation_user_pk: Option<UserPk>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl DeprecatedAction {
    pub fn assemble(id: ActionId, content: DeprecatedActionContentV1) -> Self {
        Self {
            id,
            creation_user_pk: content.creation_user_pk,
            timestamp: content.timestamp,
        }
    }

    implement_add_edge_to!(
        source_id: ActionId,
        destination_id: ActionPrototypeId,
        add_fn: add_edge_to_prototype,
        discriminant: EdgeWeightKindDiscriminants::ActionPrototype,
        result: DeprecatedActionResult,
    );

    pub async fn upsert(
        ctx: &DalContext,
        prototype_id: ActionPrototypeId,
        component_id: ComponentId,
    ) -> DeprecatedActionResult<Self> {
        for action in Self::for_component(ctx, component_id).await? {
            if action.prototype(ctx).await?.id == prototype_id {
                return Ok(action);
            }
        }

        let timestamp = Timestamp::now();

        let actor_user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => Some(*user_pk),
            _ => None,
        };

        let content = DeprecatedActionContentV1 {
            timestamp,
            creation_user_pk: actor_user_pk,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(DeprecatedActionContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Action(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot.add_node(node_weight.to_owned()).await?;

        let content_node_weight =
            node_weight.get_content_node_weight_of_kind(ContentAddressDiscriminants::Action)?;
        let action = Self::assemble(content_node_weight.id().into(), content);

        Self::add_edge_to_prototype(
            ctx,
            action.id,
            prototype_id,
            EdgeWeightKind::ActionPrototype,
        )
        .await?;

        Component::add_edge_to_deprecated_action(
            ctx,
            component_id,
            action.id,
            EdgeWeightKind::Action,
        )
        .await?;

        WsEvent::action_added(ctx, component_id, id.into())
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(action)
    }

    pub async fn delete(self, ctx: &DalContext) -> DeprecatedActionResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let change_set = ctx.change_set()?;
        workspace_snapshot
            .remove_node_by_id(change_set, self.id)
            .await?;
        Ok(())
    }

    pub async fn get_by_id(ctx: &DalContext, id: ActionId) -> DeprecatedActionResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let ulid: ::si_events::ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash: ContentHash = node_weight.content_hash();

        let content: DeprecatedActionContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let DeprecatedActionContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    pub async fn component(&self, ctx: &DalContext) -> DeprecatedActionResult<Component> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(self.id, EdgeWeightKindDiscriminants::Action)
            .await?
            .pop()
            .ok_or(DeprecatedActionError::ComponentNotFoundFor(self.id))?;
        let node_weight = workspace_snapshot
            .get_node_weight(node)
            .await?
            .get_component_node_weight()?;
        let content_hash = node_weight.content_hash();

        let content = ctx
            .layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or(DeprecatedActionError::ComponentNotFoundFor(self.id))?;

        let ComponentContent::V1(inner) = content;

        let component = Component::assemble(&node_weight, inner);
        Ok(component)
    }

    pub async fn prototype(&self, ctx: &DalContext) -> DeprecatedActionResult<ActionPrototype> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                self.id,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?
            .pop()
            .ok_or(DeprecatedActionError::PrototypeNotFoundFor(self.id))?;
        let node_weight = workspace_snapshot.get_node_weight(node).await?;
        let content_hash = node_weight.content_hash();

        let content = ctx
            .layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or(DeprecatedActionError::PrototypeNotFoundFor(self.id))?;

        let ActionPrototypeContent::V1(inner) = content;

        let prototype = ActionPrototype::assemble(node_weight.id().into(), inner);
        Ok(prototype)
    }

    pub async fn for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DeprecatedActionResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let nodes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::Action,
            )
            .await?;
        let mut node_weights = Vec::with_capacity(nodes.len());
        let mut content_hashes = Vec::with_capacity(nodes.len());
        for node in nodes {
            let weight = workspace_snapshot.get_node_weight(node).await?;
            content_hashes.push(weight.content_hash());
            node_weights.push(weight);
        }

        let content_map: HashMap<ContentHash, DeprecatedActionContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(&content_hashes)
            .await?;

        let mut actions = Vec::with_capacity(node_weights.len());
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let DeprecatedActionContent::V1(inner) = content;

                    actions.push(Self::assemble(node_weight.id().into(), inner.clone()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }
        Ok(actions)
    }

    /// A read-only method that assembles a flattened graph of [`Actions`] that will run when
    /// the change set is applied to head.
    pub async fn build_graph(
        ctx: &DalContext,
    ) -> DeprecatedActionResult<HashMap<ActionId, DeprecatedActionBag>> {
        let mut actions_by_id: HashMap<ActionId, (DeprecatedAction, ComponentId)> = HashMap::new();
        let mut actions_by_component: HashMap<ComponentId, Vec<DeprecatedAction>> = HashMap::new();
        let graph = Component::build_graph(ctx).await?;
        let mut actions_graph: HashMap<ActionId, (ComponentId, ActionKind, Vec<ActionId>)> =
            HashMap::new();

        let mut parents_graph = Vec::new();

        for (id, parent_ids) in graph {
            let component = Component::get_by_id(ctx, id).await?;

            // XXX: `is_destroyed` in the old engine should be "doesn't exist in the graph anymore"
            // in the current/new engine, but we should double check that's what this was expecting
            // to guard against.
            //
            // if component.is_destroyed() {
            //     continue;
            // }

            let actions = Self::for_component(ctx, id).await?;
            actions_by_component
                .entry(id)
                .or_default()
                .extend(actions.clone());

            let mut actions_by_kind: HashMap<ActionKind, Vec<DeprecatedAction>> = HashMap::new();
            for action in actions {
                actions_by_id.insert(action.id, (action.clone(), component.id()));

                let prototype = action.prototype(ctx).await?;
                actions_by_kind
                    .entry(prototype.kind)
                    .or_default()
                    .push(action);
            }

            let action_ids_by_kind = |kind: ActionKind| {
                actions_by_kind
                    .get(&kind)
                    .cloned()
                    .into_iter()
                    .flatten()
                    .map(|a| a.id)
            };
            let has_resource = component.resource(ctx).await?.payload.is_some();

            // Figure out internal dependencies for actions of this component
            //
            //
            // Note (FIXME/TODO): we assume actions of the same kind in the same component
            // are parallelizable, we should have some way to enable serialization and infer
            // order since they may be dependent on eachother, but there is nothing exposed about it
            for (kind, actions) in &actions_by_kind {
                for action in actions {
                    actions_graph
                        .entry(action.id)
                        .or_insert_with(|| (component.id(), *kind, Vec::new()));

                    // Action kind order is Initial Deletion -> Creation -> Others -> Final Deletion
                    // Initial deletions happen if there is a resource and a create action, so it deletes before creating
                    match kind {
                        ActionKind::Create => {
                            if has_resource {
                                let ids = action_ids_by_kind(ActionKind::Delete);
                                actions_graph
                                    .entry(action.id)
                                    .or_insert_with(|| (component.id(), *kind, Vec::new()))
                                    .2
                                    .extend(ids);
                            }
                        }
                        ActionKind::Delete => {
                            // If there is a resource and a create, this is a initial deletion, so no parent
                            if !has_resource || action_ids_by_kind(ActionKind::Create).count() == 0
                            {
                                // Every other action kind is a parent
                                let ids = actions_by_kind
                                    .iter()
                                    .filter(|(k, _)| **k != ActionKind::Delete)
                                    .flat_map(|(_, a)| a)
                                    .map(|a| a.id);
                                actions_graph
                                    .entry(action.id)
                                    .or_insert_with(|| (component.id(), *kind, Vec::new()))
                                    .2
                                    .extend(ids);
                            }
                        }
                        ActionKind::Refresh | ActionKind::Other => {
                            // If there is a resource and a create, delete actions will be initial, so our parent
                            if has_resource && action_ids_by_kind(ActionKind::Create).count() > 0 {
                                let ids = action_ids_by_kind(ActionKind::Delete);
                                actions_graph
                                    .entry(action.id)
                                    .or_insert_with(|| (component.id(), *kind, Vec::new()))
                                    .2
                                    .extend(ids);
                            }

                            let ids = action_ids_by_kind(ActionKind::Create);
                            actions_graph
                                .entry(action.id)
                                .or_insert_with(|| (component.id(), *kind, Vec::new()))
                                .2
                                .extend(ids);
                        }
                    }
                }
            }
            parents_graph.push((id, parent_ids, actions_by_kind));
        }

        for (id, parent_ids, actions_by_kind) in parents_graph {
            let component = Component::get_by_id(ctx, id).await?;
            for parent_id in parent_ids {
                let parent_component = Component::get_by_id(ctx, parent_id).await?;

                // XXX: `is_destroyed` in the old engine should be "doesn't exist in the graph anymore"
                // in the current/new engine, but we should double check that's what this was expecting
                // to guard against.
                //
                // if parent_component.is_destroyed() {
                //     continue;
                // }

                let parent_actions = actions_by_component
                    .get(&parent_component.id())
                    .cloned()
                    .unwrap_or_default();
                for (kind, actions) in &actions_by_kind {
                    for action in actions {
                        actions_graph
                            .entry(action.id)
                            .or_insert_with(|| (component.id(), *kind, Vec::new()))
                            .2
                            .extend(parent_actions.iter().map(|a| a.id));
                    }
                }
            }
        }

        let mut actions_bag_graph: HashMap<ActionId, DeprecatedActionBag> = HashMap::new();
        for (id, (component_id, kind, parents)) in actions_graph {
            actions_bag_graph.insert(
                id,
                DeprecatedActionBag {
                    component_id,
                    kind,
                    action: actions_by_id
                        .get(&id)
                        .ok_or(DeprecatedActionError::NotFound(id))?
                        .clone()
                        .0,
                    parents,
                },
            );
        }

        // Deletions require the reverse order
        let mut reversed_parents: HashMap<ActionId, Vec<ActionId>> = HashMap::new();

        for bag in actions_bag_graph.values() {
            if bag.kind == ActionKind::Delete {
                for parent_id in &bag.parents {
                    if let Some((_parent, component_id)) = actions_by_id.get(parent_id) {
                        if *component_id != bag.component_id {
                            reversed_parents
                                .entry(*parent_id)
                                .or_default()
                                .push(bag.action.id);
                        }
                    }
                }
            }
        }

        for bag in actions_bag_graph.values_mut() {
            if bag.kind == ActionKind::Delete {
                bag.parents.clear();
            }

            bag.parents.extend(
                reversed_parents
                    .get(&bag.action.id)
                    .cloned()
                    .unwrap_or_default(),
            );
        }

        Ok(actions_bag_graph)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeprecatedActionAddedPayload {
    component_id: ComponentId,
    action_id: ActionId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeprecatedActionRemovedPayload {
    component_id: ComponentId,
    action_id: ActionId,
    change_set_id: ChangeSetId,
}

impl WsEvent {
    pub async fn action_added(
        ctx: &DalContext,
        component_id: ComponentId,
        action_id: ActionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::DeprecatedActionAdded(DeprecatedActionAddedPayload {
                component_id,
                action_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn action_removed(
        ctx: &DalContext,
        component_id: ComponentId,
        action_id: ActionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::DeprecatedActionRemoved(DeprecatedActionRemovedPayload {
                component_id,
                action_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }
}
