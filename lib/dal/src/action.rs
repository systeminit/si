use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    diagram::summary_diagram::{SummaryDiagramComponent, SummaryDiagramError},
    impl_standard_model, pk, standard_model, standard_model_accessor_ro, ActionKind,
    ActionPrototype, ActionPrototypeError, ActionPrototypeId, ChangeSetPk, Component,
    ComponentError, ComponentId, DalContext, HistoryActor, HistoryEventError, Node, NodeError,
    StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError, UserPk, Visibility,
    WsEvent, WsEventError, WsEventResult, WsPayload,
};

const FIND_FOR_CHANGE_SET: &str = include_str!("./queries/action/find_for_change_set.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("history event: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("in head")]
    InHead,
    #[error(transparent)]
    Node(#[from] NodeError),
    #[error("action not found: {0}")]
    NotFound(ActionId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("action prototype not found: {0}")]
    PrototypeNotFound(ActionPrototypeId),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("summary diagram error: {0}")]
    SummaryDiagram(#[from] SummaryDiagramError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ActionResult<T> = Result<T, ActionError>;

pk!(ActionPk);
pk!(ActionId);

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ActionBag {
    pub action: Action,
    pub kind: ActionKind,
    pub parents: Vec<ActionId>,
}

// An Action joins an `ActionPrototype` to a `ComponentId` in a `ChangeSetPk`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pk: ActionPk,
    id: ActionId,
    action_prototype_id: ActionPrototypeId,
    // Change set is a field so head doesn't get cluttered with actions to it and the original
    // change set pk is lost on apply
    change_set_pk: ChangeSetPk,
    component_id: ComponentId,
    creation_user_id: Option<UserPk>,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Action,
    pk: ActionPk,
    id: ActionId,
    table_name: "actions",
    history_event_label_base: "action",
    history_event_message_name: "Action Prototype"
}

impl Action {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        prototype_id: ActionPrototypeId,
        component_id: ComponentId,
    ) -> ActionResult<Self> {
        if ctx.visibility().change_set_pk.is_none() {
            return Err(ActionError::InHead);
        }

        let actor_user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => Some(*user_pk),
            _ => None,
        };

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM action_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &prototype_id,
                    &component_id,
                    &actor_user_pk,
                ],
            )
            .await?;
        let object: Action = standard_model::finish_create_from_row(ctx, row).await?;

        WsEvent::action_added(ctx, component_id, *object.id())
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(object)
    }

    pub async fn find_for_change_set(ctx: &DalContext) -> ActionResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CHANGE_SET,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &ctx.visibility().change_set_pk,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn prototype(&self, ctx: &DalContext) -> ActionResult<ActionPrototype> {
        ActionPrototype::get_by_id(ctx, self.action_prototype_id())
            .await?
            .ok_or(ActionError::PrototypeNotFound(*self.action_prototype_id()))
    }

    pub async fn component(&self, ctx: &DalContext) -> ActionResult<Component> {
        Component::get_by_id(ctx, self.component_id())
            .await?
            .ok_or(ActionError::ComponentNotFound(*self.component_id()))
    }

    pub async fn order(ctx: &DalContext) -> ActionResult<HashMap<ActionId, ActionBag>> {
        let actions_by_id: HashMap<ActionId, Action> = Self::find_for_change_set(ctx)
            .await?
            .into_iter()
            .map(|a| (*a.id(), a))
            .collect();

        let mut actions_by_component: HashMap<ComponentId, Vec<Action>> = HashMap::new();
        for action in actions_by_id.values() {
            actions_by_component
                .entry(*action.component_id())
                .or_default()
                .push(action.clone());
        }

        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        let nodes_graph = Node::build_graph(ctx, false).await?;
        let mut actions_graph: HashMap<ActionId, (ActionKind, Vec<ActionId>)> = HashMap::new();

        for (node_id, parent_ids) in nodes_graph {
            let node = Node::get_by_id(ctx_with_deleted, &node_id)
                .await?
                .ok_or(NodeError::NotFound(node_id))?;
            let component = node
                .component(ctx_with_deleted)
                .await?
                .ok_or(NodeError::ComponentIsNone)?;

            if component.is_destroyed() {
                continue;
            }

            let actions = actions_by_component
                .get(component.id())
                .cloned()
                .unwrap_or_default();
            let mut actions_by_kind: HashMap<ActionKind, Vec<Action>> = HashMap::new();
            for action in actions {
                let prototype = action.prototype(ctx).await?;
                actions_by_kind
                    .entry(*prototype.kind())
                    .or_default()
                    .push(action);
            }

            let action_ids_by_kind = |kind: ActionKind| {
                actions_by_kind
                    .get(&kind)
                    .cloned()
                    .into_iter()
                    .flatten()
                    .map(|a| *a.id())
            };
            let has_resource = if let Some(summary) =
                SummaryDiagramComponent::get_for_component_id(ctx, *component.id()).await?
            {
                summary.has_resource()
            } else {
                component.resource(ctx).await?.payload.is_some()
            };

            // Figure out internal dependencies for actions of this component
            //
            //
            // Note (FIXME/TODO): we assume actions of the same kind in the same component
            // are parallelizable, we should have some way to enable serialization and infer
            // order since they may be dependent on eachother, but there is nothing exposed about it
            for (kind, actions) in &actions_by_kind {
                for action in actions {
                    actions_graph
                        .entry(*action.id())
                        .or_insert_with(|| (*kind, Vec::new()));

                    // Action kind order is Initial Deletion -> Creation -> Others -> Final Deletion
                    // Initial deletions happen if there is a resource and a create action, so it deletes before creating
                    match kind {
                        ActionKind::Create => {
                            if has_resource {
                                let ids = action_ids_by_kind(ActionKind::Delete);
                                actions_graph
                                    .entry(*action.id())
                                    .or_insert_with(|| (*kind, Vec::new()))
                                    .1
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
                                    .map(|a| *a.id());
                                actions_graph
                                    .entry(*action.id())
                                    .or_insert_with(|| (*kind, Vec::new()))
                                    .1
                                    .extend(ids);
                            }
                        }
                        ActionKind::Refresh | ActionKind::Other => {
                            // If there is a resource and a create, delete actions will be initial, so our parent
                            if has_resource && action_ids_by_kind(ActionKind::Create).count() > 0 {
                                let ids = action_ids_by_kind(ActionKind::Delete);
                                actions_graph
                                    .entry(*action.id())
                                    .or_insert_with(|| (*kind, Vec::new()))
                                    .1
                                    .extend(ids);
                            }

                            let ids = action_ids_by_kind(ActionKind::Create);
                            actions_graph
                                .entry(*action.id())
                                .or_insert_with(|| (*kind, Vec::new()))
                                .1
                                .extend(ids);
                        }
                    }
                }
            }

            for parent_node_id in parent_ids {
                let parent_node = Node::get_by_id(ctx_with_deleted, &parent_node_id)
                    .await?
                    .ok_or(NodeError::NotFound(parent_node_id))?;
                let parent_component = parent_node
                    .component(ctx_with_deleted)
                    .await?
                    .ok_or(NodeError::ComponentIsNone)?;

                if parent_component.is_destroyed() {
                    continue;
                }

                let parent_actions = actions_by_component
                    .get(parent_component.id())
                    .cloned()
                    .unwrap_or_default();
                for (kind, actions) in &actions_by_kind {
                    for action in actions {
                        actions_graph
                            .entry(*action.id())
                            .or_insert_with(|| (*kind, Vec::new()))
                            .1
                            .extend(parent_actions.iter().map(|a| *a.id()));
                    }
                }
            }
        }

        let mut actions_bag_graph: HashMap<ActionId, ActionBag> = HashMap::new();
        for (id, (kind, parents)) in actions_graph {
            actions_bag_graph.insert(
                id,
                ActionBag {
                    kind,
                    action: actions_by_id
                        .get(&id)
                        .ok_or(ActionError::NotFound(id))?
                        .clone(),
                    parents,
                },
            );
        }

        // Deletions require the reverse order
        let mut reversed_parents: HashMap<ActionId, Vec<ActionId>> = HashMap::new();

        for bag in actions_bag_graph.values() {
            if bag.kind == ActionKind::Delete {
                for parent_id in &bag.parents {
                    if let Some(parent) = actions_by_id.get(parent_id) {
                        if parent.component_id != bag.action.component_id {
                            reversed_parents
                                .entry(*parent_id)
                                .or_default()
                                .push(*bag.action.id());
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
                    .get(bag.action.id())
                    .cloned()
                    .unwrap_or_default(),
            );
        }

        Ok(actions_bag_graph)
    }

    standard_model_accessor_ro!(action_prototype_id, ActionPrototypeId);
    standard_model_accessor_ro!(change_set_pk, ChangeSetPk);
    standard_model_accessor_ro!(component_id, ComponentId);
    standard_model_accessor_ro!(creation_user_id, Option<UserPk>);
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionAddedPayload {
    component_id: ComponentId,
    action_id: ActionId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionRemovedPayload {
    component_id: ComponentId,
    action_id: ActionId,
    change_set_pk: ChangeSetPk,
}

impl WsEvent {
    pub async fn action_added(
        ctx: &DalContext,
        component_id: ComponentId,
        action_id: ActionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ActionAdded(ActionAddedPayload {
                component_id,
                action_id,
                change_set_pk: ctx.visibility().change_set_pk,
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
            WsPayload::ActionRemoved(ActionRemovedPayload {
                component_id,
                action_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }
}
