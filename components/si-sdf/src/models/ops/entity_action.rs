use crate::data::{Connection, Db, REQWEST};
use crate::models::{
    insert_model, Edge, EdgeKind, Entity, Node, OpError, OpResult, Resource, ResourceHealth,
    ResourceStatus, SiChangeSet, SiChangeSetEvent, SiOp, SiStorable,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntityActionRequest {
    pub action: String,
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
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
        insert_model(db, nats, &op.id, &op).await?;
        Ok(op)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub async fn apply(
        &self,
        db: &Db,
        nats: &Connection,
        hypothetical: bool,
        to: &mut serde_json::Value,
    ) -> OpResult<()> {
        if self.skip() {
            return Ok(());
        }

        let successors =
            Edge::all_successor_edges_by_object_id(db, EdgeKind::Configures, &self.to_id).await?;
        let mut entity_successors = Vec::new();
        for edge in successors.iter() {
            tracing::warn!(?edge, ?self.to_id, "what we're trying to do here...");
            let node = Node::get(
                db,
                &edge.head_vertex.node_id,
                &self.si_storable.billing_account_id,
            )
            .await?;
            let entity: serde_json::Value = if let Ok(entity) = node
                .get_object_projection(db, &self.si_change_set.change_set_id)
                .await
            {
                entity
            } else {
                node.get_head_object(db).await?
            };
            entity_successors.push(entity);
        }

        let predecessors =
            Edge::all_predecessor_edges_by_object_id(db, EdgeKind::Configures, &self.to_id).await?;
        let mut entity_predecessors = Vec::new();
        for edge in predecessors.iter() {
            tracing::warn!(?edge, ?self.to_id, "what we're trying to do here predecessors...");
            let node = Node::get(
                db,
                &edge.tail_vertex.node_id,
                &self.si_storable.billing_account_id,
            )
            .await?;
            let entity: serde_json::Value = if let Ok(entity) = node
                .get_object_projection(db, &self.si_change_set.change_set_id)
                .await
            {
                entity
            } else {
                node.get_head_object(db).await?
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
            &self.to_id,
            &self.action,
            &self.system_id,
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
            &self.system_id,
            node_id,
            entity_id,
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            self.si_storable.created_by_user_id.clone(),
        )
        .await?;

        // TODO: Next up is to take the resource update, and see if there is a resource for
        // this node. If there is, update it. If there is not, create it. Then integrate
        // the resources into the UI.
        //
        // Then, for each action we are asked to take, validate that the nodeId is in the
        // successors list (you are not allowed to notify predecessors). If it is, create
        // a new OpEntityAction, save it, and apply it immediately.

        Ok(())
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
    node_id: String,
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
