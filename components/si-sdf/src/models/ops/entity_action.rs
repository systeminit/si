use std::future::Future;
use std::pin::Pin;

use crate::data::{Connection, Db, REQWEST};
use crate::models::{
    get_base_object, insert_model, Edge, EdgeKind, Entity, Node, OpError, OpResult, Resource,
    ResourceHealth, ResourceStatus, SiChangeSet, SiChangeSetEvent, SiOp, SiStorable,
};
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
    pub fn new(
        db: Db,
        nats: Connection,
        to_id: String,
        action: String,
        system_id: String,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> Pin<Box<dyn Future<Output = OpResult<Self>> + Send>> {
        Box::pin(async move {
            let si_storable = SiStorable::new(
                &db,
                "opEntityAction",
                billing_account_id.clone(),
                organization_id,
                workspace_id,
                Some(created_by_user_id),
            )
            .await?;

            let id = si_storable.object_id.clone();

            let si_change_set = SiChangeSet::new(
                &db,
                &nats,
                change_set_id,
                edit_session_id,
                &id,
                billing_account_id,
                SiChangeSetEvent::Operation,
            )
            .await?;

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
            insert_model(&db, &nats, &op.id, &op).await?;
            Ok(op)
        })
    }

    pub async fn new_no_persist(
        db: &Db,
        nats: &Connection,
        to_id: impl Into<String>,
        action: impl Into<String>,
        system_id: impl Into<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> OpResult<Self> {
        let to_id = to_id.into();
        let action = action.into();
        let system_id = system_id.into();
        let si_storable = SiStorable::new(
            db,
            "opEntityAction",
            billing_account_id.clone(),
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let si_change_set = SiChangeSet::new(
            db,
            nats,
            change_set_id,
            edit_session_id,
            &id,
            billing_account_id,
            SiChangeSetEvent::Operation,
        )
        .await?;

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

    pub fn apply(
        &self,
        db: &Db,
        nats: &Connection,
        hypothetical: bool,
        to: &mut serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = OpResult<()>> + Send>> {
        let this_action = self.clone();
        let db = db.clone();
        let nats = nats.clone();
        let to = to.clone();
        Box::pin(async move {
            if this_action.skip() {
                return Ok(());
            }

            let successors = Edge::all_successor_edges_by_object_id(
                &db,
                EdgeKind::Configures,
                &this_action.to_id,
            )
            .await?;
            let mut entity_successors = Vec::new();
            for edge in successors.iter() {
                tracing::warn!(?edge, ?this_action.to_id, "what we're trying to do here...");
                let node = Node::get(
                    &db,
                    &edge.head_vertex.node_id,
                    &this_action.si_storable.billing_account_id,
                )
                .await?;
                let entity: serde_json::Value = if let Ok(entity) = node
                    .get_object_projection(&db, &this_action.si_change_set.change_set_id)
                    .await
                {
                    entity
                } else {
                    node.get_head_object(&db).await?
                };
                entity_successors.push(entity);
            }

            let predecessors = Edge::all_predecessor_edges_by_object_id(
                &db,
                EdgeKind::Configures,
                &this_action.to_id,
            )
            .await?;
            let mut entity_predecessors = Vec::new();
            for edge in predecessors.iter() {
                tracing::warn!(?edge, ?this_action.to_id, "what we're trying to do here predecessors...");
                let node = Node::get(
                    &db,
                    &edge.tail_vertex.node_id,
                    &this_action.si_storable.billing_account_id,
                )
                .await?;
                let entity: serde_json::Value = if let Ok(entity) = node
                    .get_object_projection(&db, &this_action.si_change_set.change_set_id)
                    .await
                {
                    entity
                } else {
                    node.get_head_object(&db).await?
                };
                entity_predecessors.push(entity);
            }

            let action_request_entities = ActionRequestEntity {
                predecessors: entity_predecessors,
                successors: entity_successors,
            };

            let action_request_resources = ActionRequestResources {
                predecessors: vec![],
                successors: vec![],
            };

            let action_request = ActionRequest::new(
                &this_action.to_id,
                &this_action.action,
                &this_action.system_id,
                hypothetical,
                action_request_entities,
                action_request_resources,
                to.clone(),
            );

            let response = run_action(action_request).await?;
            tracing::warn!(?response, "i dispatched your action!");

            let node_id = to["nodeId"].as_str().ok_or(OpError::Missing("node_id"))?;
            let entity_id = to["id"].as_str().ok_or(OpError::Missing("id"))?;

            Resource::from_update(
                &db,
                &nats,
                response.resource.state,
                response.resource.status,
                response.resource.health,
                &this_action.system_id,
                node_id,
                entity_id,
                this_action.si_storable.billing_account_id.clone(),
                this_action.si_storable.organization_id.clone(),
                this_action.si_storable.workspace_id.clone(),
                this_action.si_storable.created_by_user_id.clone(),
            )
            .await?;

            for action in response.actions.iter() {
                let new_action = OpEntityAction::new_no_persist(
                    &db,
                    &nats,
                    &action.entity_id,
                    &action.action,
                    &this_action.system_id,
                    this_action.si_storable.billing_account_id.clone(),
                    this_action.si_storable.organization_id.clone(),
                    this_action.si_storable.workspace_id.clone(),
                    this_action.si_change_set.change_set_id.clone(),
                    this_action.si_change_set.edit_session_id.clone(),
                    this_action
                        .si_storable
                        .created_by_user_id
                        .as_ref()
                        .unwrap()
                        .clone(),
                )
                .await?;
                let mut object = get_base_object(
                    &db,
                    &action.entity_id,
                    &this_action.si_change_set.change_set_id,
                )
                .await?;

                new_action
                    .apply(&db, &nats, hypothetical, &mut object)
                    .await?;
            }

            Ok(())
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequestEntity {
    predecessors: Vec<serde_json::Value>,
    successors: Vec<serde_json::Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequestResources {
    predecessors: Vec<serde_json::Value>,
    successors: Vec<serde_json::Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    to_id: String,
    action: String,
    system_id: String,
    hypothetical: bool,
    entities: ActionRequestEntity,
    resources: ActionRequestResources,
    entity: serde_json::Value,
}

impl ActionRequest {
    pub fn new(
        to_id: impl Into<String>,
        action: impl Into<String>,
        system_id: impl Into<String>,
        hypothetical: bool,
        entities: ActionRequestEntity,
        resources: ActionRequestResources,
        entity: serde_json::Value,
    ) -> ActionRequest {
        let to_id = to_id.into();
        let action = action.into();
        let system_id = system_id.into();
        ActionRequest {
            to_id,
            action,
            system_id,
            hypothetical,
            entities,
            resources,
            entity,
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

pub async fn run_action(action_request: ActionRequest) -> OpResult<ActionReply> {
    let res = REQWEST
        .post("http://localhost:5157/action")
        .json(&action_request)
        .send()
        .await?;
    let action_reply: ActionReply = res.json().await?;
    Ok(action_reply)
}
