use chrono::Utc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, trace, warn};

use crate::data::{Connection, Db, REQWEST};
use crate::models::{
    get_model, insert_model, upsert_model, Edge, EdgeError, EdgeKind, Entity, EntityError,
    EventLog, EventLogError, EventLogLevel, ModelError, OpReply, OpRequest, Resource,
    ResourceError, ResourceHealth, ResourceStatus, SiChangeSetError, SiStorable, SiStorableError,
    System, SystemError, Vertex,
};

use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error with linked entity: {0}")]
    Entity(#[from] EntityError),
    #[error("error with linked system: {0}")]
    System(#[from] SystemError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("no head object found; logic error")]
    NoHead,
    #[error("no projection object found")]
    NoProjection,
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("no object id; bug!")]
    NoObjectId,
    #[error("entity nodes require at least one system")]
    EntityRequiresSystem,
    #[error("event log error: {0}")]
    EventLog(#[from] EventLogError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

pub type NodeResult<T> = Result<T, NodeError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
    pub kind: NodeKind,
    pub object_type: String,
    pub organization_id: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_ids: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: Node,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchIncludeSystemRequest {
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchIncludeSystemReply {
    pub edge: Edge,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchConfiguredByRequest {
    pub node_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchConfiguredByReply {
    pub edge: Edge,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchSetPositionRequest {
    pub context: String,
    pub position: Position,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchSetPositionReply {
    pub context: String,
    pub position: Position,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceRequest {
    pub system_id: String,
    pub change_set_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceReply {
    pub resource: Resource,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncPredecessor {
    pub entity: Entity,
    pub resource: Resource,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceRequest<'a> {
    pub system_id: &'a str,
    pub node: &'a Node,
    pub entity: &'a Entity,
    pub resource: &'a Resource,
    pub predecessors: Vec<VeritechSyncPredecessor>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceUpdate {
    pub state: serde_json::Value,
    pub status: ResourceStatus,
    pub health: ResourceHealth,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceReply {
    pub resource: VeritechSyncResourceUpdate,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchOp {
    IncludeSystem(PatchIncludeSystemRequest),
    ConfiguredBy(PatchConfiguredByRequest),
    SetPosition(PatchSetPositionRequest),
    SyncResource(SyncResourceRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
    IncludeSystem(PatchIncludeSystemReply),
    ConfiguredBy(PatchConfiguredByReply),
    SetPosition(PatchSetPositionReply),
    SyncResource(SyncResourceReply),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    pub op: PatchOp,
    pub organization_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPatchRequest {
    pub op: OpRequest,
    pub organization_id: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ObjectPatchReply {
    Op(OpReply),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Position {
    x: u64,
    y: u64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum NodeKind {
    Entity,
    System,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_kind_string = match self {
            NodeKind::System => "system",
            NodeKind::Entity => "entity",
        };
        write!(f, "{}", node_kind_string)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub positions: HashMap<String, Position>,
    pub kind: NodeKind,
    pub object_type: String,
    pub si_storable: SiStorable,
}

impl Node {
    #[tracing::instrument(level = "trace")]
    pub async fn new(
        db: Db,
        nats: Connection,
        name: Option<String>,
        kind: NodeKind,
        object_type: impl Into<String> + std::fmt::Debug,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: Option<String>,
        system_ids: Option<Vec<String>>,
    ) -> NodeResult<Node> {
        if kind == NodeKind::Entity
            && (system_ids.is_none() || system_ids.as_ref().unwrap().len() == 0)
        {
            return Err(NodeError::EntityRequiresSystem);
        }
        let name = crate::models::generate_name(name);
        let object_type = object_type.into();
        let si_storable = SiStorable::new(
            &db,
            "node",
            billing_account_id.clone(),
            organization_id.clone(),
            workspace_id.clone(),
            created_by_user_id.clone(),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let node = Node {
            id: id.clone(),
            positions: HashMap::new(),
            kind,
            object_type: object_type.clone(),
            si_storable,
        };
        insert_model(&db, &nats, &node.id, &node).await?;

        EventLog::new(
            &db,
            &nats,
            format!("created {} node named {}", &object_type, &name),
            serde_json::json![&node],
            EventLogLevel::Info,
            billing_account_id.clone(),
            organization_id.clone(),
            workspace_id.clone(),
            created_by_user_id.clone(),
        )
        .await?;

        match node.kind {
            NodeKind::Entity => {
                let system_ids = system_ids.unwrap();
                Entity::new(
                    db.clone(),
                    nats.clone(),
                    Some(name),
                    None,
                    id,
                    object_type,
                    false,
                    billing_account_id,
                    organization_id,
                    workspace_id,
                    change_set_id.clone(),
                    edit_session_id,
                    created_by_user_id,
                    system_ids.clone(),
                )
                .await?;
                for system_id in system_ids.iter() {
                    node.sync_resource(&db, &nats, system_id, Some(change_set_id.clone()))
                        .await?;
                }
            }
            NodeKind::System => {
                System::new(
                    &db,
                    &nats,
                    Some(name),
                    None,
                    id,
                    false,
                    billing_account_id,
                    organization_id,
                    workspace_id,
                    change_set_id,
                    edit_session_id,
                    created_by_user_id,
                )
                .await?;
            }
        }

        Ok(node)
    }

    pub fn set_position(&mut self, context: String, position: Position) {
        self.positions.insert(context, position);
    }

    pub async fn sync_resource(
        &self,
        db: &Db,
        nats: &Connection,
        system_id: impl Into<String>,
        change_set_id: Option<String>,
    ) -> NodeResult<Resource> {
        let system_id = system_id.into();
        let mut resource = Resource::get_by_node_id(db, &self.id, &system_id).await?;

        let entity: Entity = if let Some(ref change_set_id) = change_set_id {
            trace!(?change_set_id, "you should really be getting a change set");
            match self.get_object_projection(&db, change_set_id).await {
                Ok(entity) => entity,
                Err(_) => self.get_head_object(db).await?,
            }
        } else if let Ok(entity) = self.get_head_object(db).await {
            entity
        } else {
            let current_time = Utc::now();
            let unix_timestamp = current_time.timestamp_millis();
            let timestamp = format!("{}", current_time);
            resource.unix_timestamp = unix_timestamp;
            resource.timestamp = timestamp;
            upsert_model(db, nats, &resource.id, &resource).await?;
            return Ok(resource);
        };

        let last_resource = resource.clone();
        let this_node = self.clone();
        let this_db = db.clone();
        let this_nats = nats.clone();
        let predecessor_edges =
            Edge::direct_predecessor_edges_by_node_id(&db, EdgeKind::Configures, &self.id).await?;

        let mut predecessors: Vec<VeritechSyncPredecessor> = Vec::new();
        for edge in predecessor_edges {
            let edge_node = Node::get(
                &db,
                &edge.tail_vertex.node_id,
                &self.si_storable.billing_account_id,
            )
            .await?;
            let mut edge_entity: Entity = if let Some(ref change_set_id) = change_set_id {
                match edge_node.get_object_projection(&db, change_set_id).await {
                    Ok(edge_entity) => edge_entity,
                    Err(_) => edge_node.get_head_object(db).await?,
                }
            } else if let Ok(edge_entity) = edge_node.get_head_object(db).await {
                edge_entity
            } else {
                return Err(NodeError::NoHead);
            };
            edge_entity.update_properties_if_secret(db).await?;
            let edge_resource = Resource::get(&db, &edge_entity.id, &system_id).await?;
            let predecessor = VeritechSyncPredecessor {
                entity: edge_entity,
                resource: edge_resource,
            };
            predecessors.push(predecessor);
        }

        tokio::spawn(async move {
            let request = VeritechSyncResourceRequest {
                system_id: &system_id,
                resource: &resource,
                node: &this_node,
                entity: &entity,
                predecessors,
            };
            let res = match REQWEST
                .post("http://localhost:5157/syncResource")
                .json(&request)
                .send()
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    warn!("sync resource error: {}", e);
                    return;
                }
            };
            let sync_reply: VeritechSyncResourceReply = match res.json().await {
                Ok(sync_reply) => sync_reply,
                Err(e) => {
                    warn!("cannot deserialize sync reply: {}", e);
                    return;
                }
            };
            match resource
                .from_update_for_self(
                    &this_db,
                    &this_nats,
                    sync_reply.resource.state,
                    sync_reply.resource.status,
                    sync_reply.resource.health,
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    warn!("cannot update resource from response: {}", e);
                    return;
                }
            }
        });
        Ok(last_resource)
    }

    pub async fn configure_node(
        &self,
        db: &Db,
        nats: &Connection,
        head_node_id: impl Into<String>,
    ) -> NodeResult<Edge> {
        let head_node_id = head_node_id.into();
        let head_node = Node::get(&db, &head_node_id, &self.si_storable.billing_account_id).await?;
        let head_object_id = head_node.get_object_id(db).await?;

        let object_id = self.get_object_id(db).await?;

        let edge = Edge::new(
            db,
            nats,
            Vertex::new(&self.id, &object_id, "output", &self.si_storable.type_name),
            Vertex::new(
                head_node_id,
                head_object_id,
                "input",
                &head_node.object_type,
            ),
            false,
            EdgeKind::Configures,
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            None,
        )
        .await?;

        Ok(edge)
    }

    pub async fn configured_by(
        &self,
        db: &Db,
        nats: &Connection,
        node_id: impl Into<String>,
    ) -> NodeResult<Edge> {
        let node_id = node_id.into();
        let node = Node::get(db, node_id, &self.si_storable.billing_account_id).await?;
        let node_object_id = node.get_object_id(db).await?;

        let object_id = self.get_object_id(db).await?;
        let edge = Edge::new(
            db,
            nats,
            Vertex::new(&node.id, &node_object_id, "output", &node.object_type),
            Vertex::new(&self.id, object_id, "input", &self.object_type),
            false,
            EdgeKind::Configures,
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            None,
        )
        .await?;
        Ok(edge)
    }

    pub async fn include_in_system(
        &self,
        db: &Db,
        nats: &Connection,
        system_id: impl Into<String>,
    ) -> NodeResult<Edge> {
        let system_id = system_id.into();
        let system = System::get_any(db, &system_id).await?;
        let object_id = self.get_object_id(db).await?;
        let edge = Edge::new(
            db,
            nats,
            Vertex::new(
                &system.node_id,
                &system.id,
                "output",
                &system.si_storable.type_name,
            ),
            Vertex::new(&self.id, object_id, "input", &self.object_type),
            false,
            EdgeKind::Includes,
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            None,
        )
        .await?;
        Ok(edge)
    }

    pub async fn get_object_id(&self, db: &Db) -> NodeResult<String> {
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"{type_name}\"
            AND a.nodeId = $node_id
          LIMIT 1
        ",
            bucket = db.bucket_name,
            type_name = self.kind,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("node_id".into(), serde_json::json![&self.id]);
        let mut query_results: Vec<serde_json::Value> =
            db.query_consistent(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(NodeError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            let object_id = String::from(result["id"].as_str().ok_or(NodeError::NoObjectId)?);
            Ok(object_id)
        }
    }

    pub async fn get_head_object<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        db: &Db,
    ) -> NodeResult<T> {
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"{type_name}\"
            AND a.nodeId = $node_id
            AND a.head = true
          LIMIT 1
        ",
            bucket = db.bucket_name,
            type_name = self.kind,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("node_id".into(), serde_json::json![&self.id]);
        let mut query_results: Vec<T> = db.query_consistent(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(NodeError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get_object_projection<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        db: &Db,
        change_set_id: impl AsRef<str>,
    ) -> NodeResult<T> {
        let change_set_id = change_set_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"{type_name}\"
            AND a.siChangeSet.changeSetId = $change_set_id
            AND a.nodeId = $node_id
            AND a.head = false
          ORDER BY a.base ASC
          LIMIT 1
        ",
            bucket = db.bucket_name,
            type_name = self.kind,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("node_id".into(), serde_json::json![&self.id]);
        named_params.insert("change_set_id".into(), serde_json::json![change_set_id]);
        trace!(?named_params, ?query, "getting obj projection");
        let mut query_results: Vec<T> = db.query_consistent(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(NodeError::NoProjection)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get(
        db: &Db,
        node_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> NodeResult<Node> {
        let node_id = node_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();

        let node: Node = get_model(db, node_id, billing_account_id).await?;
        Ok(node)
    }
}
