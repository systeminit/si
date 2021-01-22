use crate::data::{NatsTxn, PgTxn};

use crate::models::{
    next_update_clock, Edge, EdgeKind, Entity, Event, Node, OpError, OpResult, Resource,
    ResourceHealth, ResourceStatus, SiChangeSet, SiChangeSetEvent, SiOp, SiStorable, UpdateClock,
};
use crate::veritech::{Veritech, VeritechError};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntityActionRequest {
    pub action: String,
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpEntityAction {
    pub id: String,
    pub to_id: String,
    pub action: String,
    pub system_id: String,
    pub si_op: SiOp,
    pub si_storable: SiStorable,
    pub si_change_set: SiChangeSet,
}

impl OpEntityAction {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        to_id: impl Into<String>,
        action: impl Into<String>,
        system_id: impl Into<String>,
        workspace_id: impl Into<String>,
        change_set_id: impl Into<String>,
        edit_session_id: impl Into<String>,
    ) -> OpResult<Self> {
        let workspace_id = workspace_id.into();
        let change_set_id = change_set_id.into();
        let edit_session_id = edit_session_id.into();
        let to_id = to_id.into();
        let action = action.into();
        let system_id = system_id.into();

        let workspace_update_clock = next_update_clock(&workspace_id).await?;
        let change_set_update_clock = next_update_clock(&change_set_id).await?;
        let override_system: Option<String> = None;

        let row = txn
                .query_one(
                    "SELECT object FROM op_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                    &[
                    &"opEntityAction",
                    &to_id,
                    &serde_json::json![{"action": action, "systemId": system_id }],
                    &override_system,
                    &change_set_id,
                    &edit_session_id,
                    &SiChangeSetEvent::Operation.to_string(),
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                    &change_set_update_clock.epoch,
                    &change_set_update_clock.update_count,
                    ],
                )
                .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn new_no_persist(
        to_id: impl Into<String>,
        action: impl Into<String>,
        system_id: impl Into<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
    ) -> OpResult<Self> {
        let to_id = to_id.into();
        let action = action.into();
        let system_id = system_id.into();
        let object_id = String::from("opEntityAction:ephemeral");
        let si_storable = SiStorable {
            type_name: String::from("opEntityAction"),
            object_id: object_id.clone(),
            billing_account_id: billing_account_id.clone(),
            organization_id: organization_id.clone(),
            workspace_id: workspace_id.clone(),
            tenant_ids: vec![
                billing_account_id.clone(),
                organization_id.clone(),
                workspace_id.clone(),
                object_id.clone(),
            ],
            created_by_user_id: None,
            update_clock: UpdateClock {
                epoch: 0,
                update_count: 0,
            },
            deleted: false,
        };
        let si_change_set = SiChangeSet {
            change_set_id: change_set_id.clone(),
            edit_session_id: edit_session_id.clone(),
            event: SiChangeSetEvent::Operation,
            order_clock: UpdateClock {
                epoch: 0,
                update_count: 0,
            },
        };

        let id = object_id.clone();

        let si_op = SiOp::new(None);

        let op = OpEntityAction {
            id,
            to_id,
            action,
            system_id,
            si_op,
            si_storable,
            si_change_set,
        };
        Ok(op)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub async fn apply(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: &Veritech,
        hypothetical: bool,
        to: &mut serde_json::Value,
        event_parent_id: Option<String>,
    ) -> OpResult<()> {
        if self.skip() {
            return Ok(());
        }

        let mut apply_stack = vec![self.clone()];
        while let Some(this_action) = apply_stack.pop() {
            let entity: Entity = serde_json::from_value(to.clone())?;
            let node: Node = Node::get(&txn, &entity.node_id).await?;
            let resource = Resource::get_any_by_entity_id(
                &txn,
                &entity.id,
                &this_action.system_id,
                &this_action.si_change_set.change_set_id,
            )
            .await?;

            // Populate Successors
            let successor_edges =
                Edge::direct_successor_edges_by_node_id(&txn, &EdgeKind::Configures, &node.id)
                    .await?;
            let mut successors: Vec<ActionRequestThunk> = Vec::new();
            for edge in successor_edges.iter() {
                let mut edge_entity: Entity = Entity::get_projection_or_head(
                    &txn,
                    &edge.head_vertex.object_id,
                    &this_action.si_change_set.change_set_id,
                )
                .await?;
                edge_entity.update_properties_if_secret(&txn).await?;

                let edge_resource = Resource::get_any_by_entity_id(
                    &txn,
                    &edge_entity.id,
                    &this_action.system_id,
                    &this_action.si_change_set.change_set_id,
                )
                .await?;
                successors.push(ActionRequestThunk {
                    entity: edge_entity,
                    resource: edge_resource,
                });
            }

            // Populate Predecessors
            let predecessor_edges =
                Edge::direct_predecessor_edges_by_node_id(&txn, &EdgeKind::Configures, &node.id)
                    .await?;
            let mut predecessors: Vec<ActionRequestThunk> = Vec::new();
            for edge in predecessor_edges.iter() {
                let mut edge_entity: Entity = Entity::get_projection_or_head(
                    &txn,
                    &edge.tail_vertex.object_id,
                    &this_action.si_change_set.change_set_id,
                )
                .await?;
                edge_entity.update_properties_if_secret(&txn).await?;
                let edge_resource = Resource::get_any_by_entity_id(
                    &txn,
                    &edge_entity.id,
                    &this_action.system_id,
                    &this_action.si_change_set.change_set_id,
                )
                .await?;
                predecessors.push(ActionRequestThunk {
                    entity: edge_entity,
                    resource: edge_resource,
                });
            }

            let mut event = Event::entity_action(
                &txn,
                &nats,
                &this_action,
                &entity,
                &this_action.system_id,
                event_parent_id.clone(),
            )
            .await?;

            let action_request = ActionRequest::new(
                &this_action.action,
                &this_action.system_id,
                node,
                entity,
                resource,
                hypothetical,
                predecessors,
                successors,
            );

            let response: ActionReply = match veritech
                .send(&txn, &nats, "/ws/action", action_request, &event)
                .await?
            {
                Some(response) => response,
                None => {
                    event.unknown(&txn, &nats).await?;
                    return Err(OpError::Veritech(VeritechError::NoReply));
                }
            };

            tracing::warn!(?response, "i dispatched your action!");

            let entity_id = to["id"].as_str().ok_or(OpError::Missing("id"))?;

            Resource::from_update(
                &txn,
                &nats,
                response.resource.state,
                response.resource.status,
                response.resource.health,
                hypothetical,
                &this_action.system_id,
                entity_id,
                &this_action.si_change_set.change_set_id,
            )
            .await?;

            for action in response.actions.iter() {
                let new_action = OpEntityAction::new_no_persist(
                    &action.entity_id,
                    &action.action,
                    &this_action.system_id,
                    this_action.si_storable.billing_account_id.clone(),
                    this_action.si_storable.organization_id.clone(),
                    this_action.si_storable.workspace_id.clone(),
                    this_action.si_change_set.change_set_id.clone(),
                    this_action.si_change_set.edit_session_id.clone(),
                )
                .await?;
                apply_stack.push(new_action);
            }
            event.success(&txn, &nats).await?;
        }
        Ok(())
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequestThunk {
    entity: Entity,
    resource: Resource,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    action: String,
    system_id: String,
    node: Node,
    entity: Entity,
    resource: Resource,
    hypothetical: bool,
    predecessors: Vec<ActionRequestThunk>,
    successors: Vec<ActionRequestThunk>,
}

impl ActionRequest {
    pub fn new(
        action: impl Into<String>,
        system_id: impl Into<String>,
        node: Node,
        entity: Entity,
        resource: Resource,
        hypothetical: bool,
        predecessors: Vec<ActionRequestThunk>,
        successors: Vec<ActionRequestThunk>,
    ) -> ActionRequest {
        let action = action.into();
        let system_id = system_id.into();
        ActionRequest {
            action,
            system_id,
            node,
            entity,
            resource,
            hypothetical,
            predecessors,
            successors,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionResourceUpdate {
    state: serde_json::Value,
    status: ResourceStatus,
    health: ResourceHealth,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionUpdate {
    action: String,
    entity_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionReply {
    resource: ActionResourceUpdate,
    actions: Vec<ActionUpdate>,
}
