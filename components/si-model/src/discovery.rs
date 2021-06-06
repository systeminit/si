use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::{NatsConn, PgPool, PgTxn};

use crate::{
    entity::Op,
    resource,
    workflow::selector::{SelectionEntry, SelectionEntryPredecessor},
    ChangeSet, ChangeSetError, Edge, EdgeError, EdgeKind, EditSession, EditSessionError, Entity,
    EntityError, Node, NodeError, Resource, ResourceError, Veritech, VeritechError, Vertex,
    WorkflowError,
};

pub const DISCOVERY_LIST: &str = include_str!("./queries/discovery_list.sql");

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("discovery request entity is not in a system, invalid request")]
    NoSystem,
    #[error("workflow error: {0}")]
    Workflow(#[from] WorkflowError),
    #[error("veritech error: {0}")]
    Veritech(#[from] VeritechError),
    #[error("changeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("editSession error: {0}")]
    EditSession(#[from] EditSessionError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
}

pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryListEntry {
    entity: Entity,
    resource: Resource,
}

pub async fn list(
    txn: &PgTxn<'_>,
    workspace_id: impl AsRef<str>,
    entity_type: impl AsRef<str>,
) -> DiscoveryResult<Vec<DiscoveryListEntry>> {
    let workspace_id = workspace_id.as_ref();
    let entity_type = entity_type.as_ref();
    let rows = txn
        .query(DISCOVERY_LIST, &[&workspace_id, &entity_type])
        .await?;

    let mut list = Vec::new();
    for row in rows.into_iter() {
        let entity_json: serde_json::Value = row.try_get("entity")?;
        let entity: Entity = serde_json::from_value(entity_json)?;
        let resource_json: serde_json::Value = row.try_get("resource")?;
        let resource: Resource = serde_json::from_value(resource_json)?;

        list.push(DiscoveryListEntry { entity, resource });
    }
    Ok(list)
}

pub async fn implementations_list(
    txn: &PgTxn<'_>,
    workspace_id: impl AsRef<str>,
    application_id: impl AsRef<str>,
    implementation_entity_types: Vec<String>,
) -> DiscoveryResult<HashMap<String, Vec<DiscoveryListEntry>>> {
    let mut reply = HashMap::new();
    let application_id = application_id.as_ref();
    let workspace_id = workspace_id.as_ref();
    let application_edges =
        Edge::direct_successor_edges_by_object_id(&txn, &EdgeKind::Includes, &application_id)
            .await?;
    for implementation_entity_type in implementation_entity_types.into_iter() {
        let impls = list(&txn, &workspace_id, &implementation_entity_type).await?;
        let filter_impls: Vec<DiscoveryListEntry> = impls
            .into_iter()
            .filter(|i| {
                application_edges
                    .iter()
                    .any(|ae| ae.head_vertex.object_id == i.entity.id)
                    != true
            })
            .collect();
        reply.insert(implementation_entity_type, filter_impls);
    }
    Ok(reply)
}

pub async fn discover(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
    workspace_id: impl Into<String>,
    entity_id: impl Into<String>,
    entity_type: impl Into<String>,
) -> DiscoveryResult<()> {
    let workspace_id = workspace_id.into();
    let entity_id = entity_id.into();
    let entity_type = entity_type.into();
    let pg = pg.clone();
    let nats_conn = nats_conn.clone();
    let veritech = veritech.clone();
    tokio::spawn(async move {
        match task_discover(
            pg,
            nats_conn,
            veritech,
            workspace_id,
            entity_id,
            entity_type,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                dbg!("who knows what happened doing discovery: {:?}", &e);
            }
        };
    });
    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryRequest<'a> {
    entity: &'a Entity,
    system: &'a Entity,
    entity_type: &'a str,
    context: Vec<SelectionEntryPredecessor>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialEntity {
    name: String,
    ops: Vec<Op>,
    entity_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverEntity {
    entity: PartialEntity,
    configures: Vec<DiscoverEntity>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryFinish {
    pub discovered: Vec<DiscoverEntity>,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryProtocol {
    Start(bool),
    Finish(DiscoveryFinish),
}

pub async fn task_discover(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    workspace_id: String,
    entity_id: String,
    entity_type: String,
) -> DiscoveryResult<()> {
    let mut conn = pg.pool.get().await?;
    let txn = conn.transaction().await?;

    // The root entity that will be doing discovery
    let entity = Entity::for_head(&txn, &entity_id).await?;

    let system =
        Entity::get_head_by_name_and_entity_type(&txn, "production", "system", &workspace_id)
            .await?
            .into_iter()
            .nth(0)
            .ok_or(DiscoveryError::NoSystem)?;

    let context = SelectionEntry::new(&txn, &entity.id, &system).await?;

    txn.commit().await?;

    let request = DiscoveryRequest {
        entity: &entity,
        system: &system,
        entity_type: &entity_type,
        context: context.context,
    };

    let (progress_tx, mut progress_rx) =
        tokio::sync::mpsc::unbounded_channel::<DiscoveryProtocol>();

    veritech
        .send_async("discover", request, progress_tx)
        .await?;
    while let Some(message) = progress_rx.recv().await {
        match message {
            DiscoveryProtocol::Start(_) => {}
            DiscoveryProtocol::Finish(finish) => {
                for discovered in finish.discovered.into_iter() {
                    let mut to_discover: Vec<(Option<Entity>, DiscoverEntity)> =
                        vec![(None, discovered)];
                    let mut index = 0;
                    while index < to_discover.len() {
                        let txn = conn.transaction().await?;
                        let nats = nats_conn.transaction();
                        let (parent, discover) = to_discover[index].clone();
                        index = index + 1;
                        let has_entity = Entity::get_head_by_name_and_entity_type(
                            &txn,
                            &discover.entity.name,
                            &discover.entity.entity_type,
                            &workspace_id,
                        )
                        .await?
                        .len()
                            > 0;
                        if !has_entity {
                            let mut change_set = ChangeSet::new(
                                &txn,
                                &nats,
                                Some(format!(
                                    "d({} {})",
                                    &discover.entity.entity_type, &discover.entity.name
                                )),
                                workspace_id.clone(),
                            )
                            .await?;
                            let mut edit_session = EditSession::new(
                                &txn,
                                &nats,
                                None,
                                change_set.id.clone(),
                                workspace_id.clone(),
                            )
                            .await?;

                            let node = Node::new(
                                &pg,
                                &txn,
                                &nats_conn,
                                &nats,
                                &veritech,
                                Some(discover.entity.name.clone()),
                                &discover.entity.entity_type,
                                workspace_id.clone(),
                                change_set.id.clone(),
                                edit_session.id.clone(),
                            )
                            .await?;

                            let mut entity = Entity::for_edit_session(
                                &txn,
                                &node.object_id,
                                &change_set.id,
                                &edit_session.id,
                            )
                            .await?;
                            entity.ops = discover.entity.ops.clone();
                            entity
                                .infer_properties_for_edit_session(
                                    &txn,
                                    &veritech,
                                    &change_set.id,
                                    &edit_session.id,
                                )
                                .await?;
                            entity
                                .save_for_edit_session(&txn, &change_set.id, &edit_session.id)
                                .await?;

                            if let Some(parent_entity) = parent {
                                let tail_vertex = Vertex::new(
                                    &entity.node_id,
                                    &entity.id,
                                    "output",
                                    &entity.entity_type,
                                );
                                let head_vertex = Vertex::new(
                                    &parent_entity.node_id,
                                    &parent_entity.id,
                                    &entity.entity_type,
                                    &parent_entity.entity_type,
                                );
                                Edge::new(
                                    &txn,
                                    &nats,
                                    tail_vertex,
                                    head_vertex,
                                    false,
                                    EdgeKind::Configures,
                                    &workspace_id,
                                )
                                .await?;
                            }

                            edit_session.save_session(&txn).await?;
                            change_set.apply(&txn).await?;
                            txn.commit().await?;

                            entity
                                .check_qualifications_for_edit_session(
                                    &pg,
                                    &nats_conn,
                                    &veritech,
                                    Some(system.id.clone()),
                                    &change_set.id,
                                    &edit_session.id,
                                )
                                .await?;

                            resource::sync_resource(&pg, &nats_conn, &veritech, &entity).await?;

                            for child in discover.configures.iter() {
                                to_discover.push((Some(entity.clone()), child.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
