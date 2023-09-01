use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    ActionKind, ActionPrototype, ActionPrototypeError, ActionPrototypeId, ChangeSetPk, Component,
    ComponentError, ComponentId, DalContext, HistoryEventError, Node, NodeError, StandardModel,
    StandardModelError, Tenancy, Timestamp, TransactionsError, Visibility, WsEvent, WsEventError,
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
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("action prototype not found: {0}")]
    PrototypeNotFound(ActionPrototypeId),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ActionResult<T> = Result<T, ActionError>;

pk!(ActionPk);
pk!(ActionId);

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
    index: i16,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        prototype_id: ActionPrototypeId,
        component_id: ComponentId,
    ) -> ActionResult<Self> {
        if ctx.visibility().change_set_pk.is_none() {
            return Err(ActionError::InHead);
        }

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM action_create_v1($1, $2, $3, $4)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &prototype_id,
                    &component_id,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;

        WsEvent::change_set_written(ctx)
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

    pub async fn sort_of_change_set(ctx: &DalContext) -> ActionResult<()> {
        let actions = Self::find_for_change_set(ctx).await?;

        let mut actions_by_component: HashMap<ComponentId, Vec<Action>> = HashMap::new();
        for action in actions {
            actions_by_component
                .entry(*action.component_id())
                .or_default()
                .push(action);
        }

        let mut initial_deletions = Vec::new();
        let mut initial_others = Vec::new();
        let mut creations = Vec::new();
        let mut final_others = Vec::new();
        let mut final_deletions = Vec::new();

        let sorted_node_ids =
            Node::list_topologically_sorted_configuration_nodes_with_stable_ordering(ctx, false)
                .await?;

        let ctx_with_deleted = &ctx.clone_with_delete_visibility();
        for sorted_node_id in sorted_node_ids {
            let sorted_node = Node::get_by_id(ctx_with_deleted, &sorted_node_id)
                .await?
                .ok_or(NodeError::NotFound(sorted_node_id))?;
            let component = sorted_node
                .component(ctx_with_deleted)
                .await?
                .ok_or(NodeError::ComponentIsNone)?;

            if component.is_destroyed() {
                continue;
            }

            let mut actions =
                if let Some(actions) = actions_by_component.get(component.id()).cloned() {
                    actions
                } else {
                    continue;
                };

            // Make them stable
            actions.sort_by_key(|a| *a.action_prototype_id());

            for action in actions {
                let prototype = action.prototype(ctx).await?;
                match prototype.kind() {
                    ActionKind::Create => {
                        creations.push(action);
                    }
                    ActionKind::Delete => {
                        if component.resource(ctx).await?.payload.is_some() {
                            initial_deletions.push(action);
                        } else {
                            final_deletions.push(action);
                        }
                    }
                    ActionKind::Refresh | ActionKind::Other => {
                        if component.resource(ctx).await?.payload.is_some() {
                            initial_others.push(action);
                        } else {
                            final_others.push(action);
                        }
                    }
                }
            }
        }

        initial_deletions.reverse();
        final_deletions.reverse();

        let mut actions = Vec::with_capacity(
            initial_deletions.len()
                + creations.len()
                + initial_others.len()
                + final_others.len()
                + final_deletions.len(),
        );
        actions.extend(initial_deletions);
        actions.extend(initial_others);
        actions.extend(creations);
        actions.extend(final_others);
        actions.extend(final_deletions);

        for (index, mut action) in actions.into_iter().enumerate() {
            action.set_index(ctx, index as i16).await?;
        }

        WsEvent::change_set_written(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }

    standard_model_accessor!(index, i16, ActionResult);
    standard_model_accessor_ro!(action_prototype_id, ActionPrototypeId);
    standard_model_accessor_ro!(change_set_pk, ChangeSetPk);
    standard_model_accessor_ro!(component_id, ComponentId);
}
