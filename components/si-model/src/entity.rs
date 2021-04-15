use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    generate_name, Edge, EdgeError, EdgeKind, Qualification, QualificationError, Resource,
    ResourceError, SiChangeSet, SiStorable, Veritech, VeritechError,
};
use si_data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};

pub mod diff;

const ENTITY_GET_HEAD_BY_NAME_AND_ENTITY_TYPE: &str =
    include_str!("./queries/entity_get_head_by_name_and_entity_type.sql");
const ENTITY_FOR_EDIT_SESSION: &str = include_str!("./queries/entity_for_edit_session.sql");
const ENTITY_FOR_CHANGE_SET: &str = include_str!("./queries/entity_for_change_set.sql");
const ENTITY_FOR_HEAD: &str = include_str!("./queries/entity_for_head.sql");

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("no head entity found; logic error")]
    NoHead,
    #[error("no override system found: {0}")]
    Override(String),
    #[error("invalid entity; missing object type")]
    MissingObjectType,
    #[error("invalid entity; missing node id")]
    MissingId,
    #[error("missing field: {0}")]
    Missing(String),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("not found")]
    NotFound,
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("malformed database entry")]
    MalformedDatabaseEntry,
    #[error("missing change set in a save where it is required")]
    MissingChangeSet,
    #[error("veritech error: {0}")]
    Veritech(#[from] VeritechError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("deadpool error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("no change set provided")]
    NoChangeSet,
    #[error("qualification error: {0}")]
    Qualification(#[from] QualificationError),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InferPropertiesPredecessor {
    pub entity: Entity,
    pub resources: Vec<Resource>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InferPropertiesRequest {
    entity_type: String,
    entity: Entity,
    predecessors: Vec<InferPropertiesPredecessor>,
    resources: Vec<Resource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InferPropertiesResponse {
    entity: Entity,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckQualificationsRequest {
    system_id: String,
    entity_type: String,
    entity: Entity,
    predecessors: Vec<InferPropertiesPredecessor>,
    resources: Vec<Resource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CheckQualificationsProtocol {
    ValidNames(Vec<String>),
    Start(String),
    Item(CheckQualificationsItem),
    Finished,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckQualificationsItem {
    name: String,
    qualified: bool,
    output: Option<String>,
    error: Option<String>,
}

pub type EntityResult<T> = Result<T, EntityError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OpType {
    Set,
    Unset,
    Tombstone,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OpSource {
    Manual,
    Expression,
    Inferred,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Op {
    op: OpType,
    source: OpSource,
    system: String,
    path: Vec<String>,
    value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpTombstone {
    op: OpType,
    source: OpSource,
    system: String,
    path: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrayMeta {
    length: u64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub node_id: String,
    pub name: String,
    pub description: String,
    pub entity_type: String,
    pub ops: Vec<Op>,
    pub tombstones: Vec<OpTombstone>,
    pub array_meta: serde_json::Value,
    pub properties: serde_json::Value,
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl Entity {
    pub async fn new(
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        _nats: &NatsTxn,
        veritech: &Veritech,
        name: Option<String>,
        description: Option<String>,
        node_id: impl AsRef<str>,
        object_type: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let object_type = object_type.as_ref();
        let node_id = node_id.as_ref();

        let name = generate_name(name);
        let description = if description.is_some() {
            description.unwrap()
        } else {
            name.clone()
        };

        let row = txn
            .query_one(
                "SELECT object FROM entity_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &name,
                    &description,
                    &object_type,
                    &node_id,
                    &change_set_id,
                    &edit_session_id,
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let mut entity: Entity = serde_json::from_value(json)?;
        let changed = entity
            .infer_properties_for_edit_session(&txn, &veritech, &change_set_id, &edit_session_id)
            .await?;
        if changed {
            Entity::calculate_properties_of_successors_for_edit_session(
                &pg,
                &nats_conn,
                &veritech,
                String::from(&entity.id),
                String::from(change_set_id),
                String::from(edit_session_id),
            )
            .await?;
        }

        Ok(entity)
    }

    pub async fn delete(&mut self) -> EntityResult<()> {
        self.si_storable.deleted = true;
        Ok(())
    }

    pub async fn update_entity_for_edit_session(
        &mut self,
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        _nats: &NatsTxn,
        veritech: &Veritech,
        change_set_id: impl Into<String>,
        edit_session_id: impl Into<String>,
    ) -> EntityResult<()> {
        let change_set_id = change_set_id.into();
        let edit_session_id = edit_session_id.into();
        self.save_for_edit_session(&txn, &change_set_id, &edit_session_id)
            .await?;
        let changed = self
            .infer_properties_for_edit_session(&txn, &veritech, &change_set_id, &edit_session_id)
            .await?;
        if changed {
            Entity::calculate_properties_of_successors_for_edit_session(
                &pg,
                &nats_conn,
                &veritech,
                String::from(&self.id),
                String::from(change_set_id),
                String::from(edit_session_id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn task_check_qualifications_for_edit_session(
        entity: Entity,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
        system_id: Option<String>,
        change_set_id: String,
        edit_session_id: String,
        workspace_id: String,
    ) -> EntityResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let system_id = system_id.unwrap_or("baseline".to_string());
        let resources = Resource::for_edit_session_by_entity_id(
            &txn,
            &entity.id,
            &change_set_id,
            &edit_session_id,
        )
        .await?;

        let predecessor_edges =
            Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &entity.id)
                .await?;
        let mut predecessors: Vec<InferPropertiesPredecessor> = Vec::new();
        for edge in predecessor_edges {
            let edge_entity = Entity::for_edit_session(
                &txn,
                &edge.tail_vertex.object_id,
                &change_set_id,
                &edit_session_id,
            )
            .await?;
            let edge_resources = Resource::for_edit_session_by_entity_id(
                &txn,
                &edge.tail_vertex.object_id,
                &change_set_id,
                &edit_session_id,
            )
            .await?;
            let predecessor = InferPropertiesPredecessor {
                entity: edge_entity,
                resources: edge_resources,
            };
            predecessors.push(predecessor);
        }
        let request = CheckQualificationsRequest {
            entity_type: entity.entity_type.clone(),
            entity: entity.clone(),
            predecessors,
            resources,
            system_id,
        };
        let (progress_tx, mut progress_rx) =
            tokio::sync::mpsc::unbounded_channel::<CheckQualificationsProtocol>();

        veritech
            .send_async("checkQualifications", request, progress_tx)
            .await?;

        txn.commit().await?;
        nats.commit().await?;

        let mut valid_names: Vec<String> = vec![];
        while let Some(message) = progress_rx.recv().await {
            match message {
                CheckQualificationsProtocol::ValidNames(names) => {
                    valid_names = names;
                }
                CheckQualificationsProtocol::Start(check_name) => {
                    dbg!(format!("starting {}", check_name));
                    let nats = nats_conn.transaction();
                    let mut storable = entity.si_storable.clone();
                    storable.type_name = "qualificationStart".to_string();
                    nats.publish(&serde_json::json!({
                        "start": check_name,
                        "entityId": entity.id.clone(),
                        "changeSetId": change_set_id.clone(),
                        "editSessionId": edit_session_id.clone(),
                        "siStorable": storable,
                    }))
                    .await?;
                    nats.commit().await?;
                }
                CheckQualificationsProtocol::Item(qual) => {
                    dbg!(format!("got a qualification: {:?}", qual));
                    let txn = conn.transaction().await?;
                    let nats = nats_conn.transaction();
                    let q = Qualification::new(
                        &txn,
                        &nats,
                        &entity.id,
                        qual.name,
                        qual.qualified,
                        qual.output,
                        qual.error,
                        &change_set_id,
                        &edit_session_id,
                        &workspace_id,
                    )
                    .await?;
                    txn.commit().await?;
                    nats.commit().await?;
                    dbg!(&q);
                }
                CheckQualificationsProtocol::Finished => {
                    dbg!("got a finished message");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn check_qualifications_for_edit_session(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        system_id: Option<String>,
        change_set_id: impl Into<String>,
        edit_session_id: impl Into<String>,
    ) -> EntityResult<bool> {
        let change_set_id = change_set_id.into();
        let edit_session_id = edit_session_id.into();
        let pg = pg.clone();
        let nats_conn = nats_conn.clone();
        let veritech = veritech.clone();
        let entity = self.clone();
        let workspace_id = entity.si_storable.workspace_id.clone();
        tokio::spawn(async move {
            match Entity::task_check_qualifications_for_edit_session(
                entity,
                pg,
                nats_conn,
                veritech,
                system_id,
                change_set_id,
                edit_session_id,
                workspace_id,
            )
            .await
            {
                Ok(()) => {}
                Err(e) => {
                    dbg!("who knows what happened checking qualifications: {:?}", &e);
                }
            }
        });
        Ok(true)
    }

    pub async fn infer_properties_for_edit_session(
        &mut self,
        txn: &PgTxn<'_>,
        veritech: &Veritech,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> EntityResult<bool> {
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let resources = Resource::for_edit_session_by_entity_id(
            &txn,
            &self.id,
            &change_set_id,
            &edit_session_id,
        )
        .await?;

        let predecessor_edges =
            Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &self.id)
                .await?;
        let mut predecessors: Vec<InferPropertiesPredecessor> = Vec::new();
        for edge in predecessor_edges {
            let edge_entity = Entity::for_edit_session(
                &txn,
                &edge.tail_vertex.object_id,
                &change_set_id,
                &edit_session_id,
            )
            .await?;
            let edge_resources = Resource::for_edit_session_by_entity_id(
                &txn,
                &edge.tail_vertex.object_id,
                &change_set_id,
                &edit_session_id,
            )
            .await?;
            let predecessor = InferPropertiesPredecessor {
                entity: edge_entity,
                resources: edge_resources,
            };
            predecessors.push(predecessor);
        }
        let request = InferPropertiesRequest {
            entity_type: self.entity_type.clone(),
            entity: self.clone(),
            predecessors,
            resources,
        };
        let response = veritech.infer_properties(request).await?;
        if self != &response.entity {
            *self = response.entity;
            self.save_for_edit_session(&txn, &change_set_id, &edit_session_id)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn save_for_edit_session(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> EntityResult<()> {
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let entity_json = serde_json::to_value(self)?;
        let _row = txn
            .query_one(
                "SELECT true FROM entity_save_for_edit_session_v1($1, $2, $3)",
                &[&entity_json, &change_set_id, &edit_session_id],
            )
            .await?;
        Ok(())
    }

    pub async fn for_edit_session(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        let entity_id = entity_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let row = txn
            .query_one(
                ENTITY_FOR_EDIT_SESSION,
                &[&entity_id, &change_set_id, &edit_session_id],
            )
            .await?;
        let object: serde_json::Value = row.try_get("object")?;
        let entity: Entity = serde_json::from_value(object)?;
        Ok(entity)
    }

    pub async fn for_change_set(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        let entity_id = entity_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let row = txn
            .query_one(ENTITY_FOR_CHANGE_SET, &[&entity_id, &change_set_id])
            .await?;
        let object: serde_json::Value = row.try_get("object")?;
        let entity: Entity = serde_json::from_value(object)?;
        Ok(entity)
    }

    pub async fn for_head(txn: &PgTxn<'_>, entity_id: impl AsRef<str>) -> EntityResult<Entity> {
        let entity_id = entity_id.as_ref();
        let row = txn.query_one(ENTITY_FOR_HEAD, &[&entity_id]).await?;
        let object: serde_json::Value = row.try_get("object")?;
        let entity: Entity = serde_json::from_value(object)?;
        Ok(entity)
    }

    pub async fn for_head_or_change_set(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: Option<&String>,
    ) -> EntityResult<Entity> {
        if let Some(change_set_id) = change_set_id {
            Entity::for_change_set(&txn, entity_id, change_set_id).await
        } else {
            Entity::for_head(&txn, entity_id).await
        }
    }

    pub async fn for_head_or_change_set_or_edit_session(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> EntityResult<Entity> {
        if let Some(edit_session_id) = edit_session_id {
            if let Some(change_set_id) = change_set_id {
                Entity::for_edit_session(&txn, &entity_id, change_set_id, edit_session_id).await
            } else {
                return Err(EntityError::NoChangeSet);
            }
        } else if let Some(change_set_id) = change_set_id {
            Entity::for_change_set(&txn, entity_id, change_set_id).await
        } else {
            Entity::for_head(&txn, entity_id).await
        }
    }

    pub async fn for_diff(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> EntityResult<Entity> {
        if let Some(_edit_session_id) = edit_session_id {
            if let Some(change_set_id) = change_set_id {
                Entity::for_change_set(&txn, &entity_id, change_set_id).await
            } else {
                return Err(EntityError::NoChangeSet);
            }
        } else {
            Entity::for_head(&txn, entity_id).await
        }
    }

    pub async fn task_calculate_properties_of_successors_for_edit_session(
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
        first_entity_id: String,
        change_set_id: String,
        edit_session_id: String,
    ) -> EntityResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;

        let mut entities_to_check = vec![first_entity_id];

        while let Some(entity_id) = entities_to_check.pop() {
            let successors =
                Edge::direct_successor_edges_by_object_id(&txn, &EdgeKind::Configures, &entity_id)
                    .await?;
            for edge in successors.iter() {
                let mut edge_entity = Entity::for_edit_session(
                    &txn,
                    &edge.head_vertex.object_id,
                    &change_set_id,
                    &edit_session_id,
                )
                .await?;
                let changed = edge_entity
                    .infer_properties_for_edit_session(
                        &txn,
                        &veritech,
                        &change_set_id,
                        &edit_session_id,
                    )
                    .await?;
                if changed {
                    edge_entity
                        .check_qualifications_for_edit_session(
                            &pg,
                            &nats_conn,
                            &veritech,
                            None,
                            change_set_id.clone(),
                            edit_session_id.clone(),
                        )
                        .await?;
                    entities_to_check.push(edge_entity.id);
                }
            }
        }
        // Probably also some nats work to do here.
        txn.commit().await?;

        Ok(())
    }

    pub async fn calculate_properties_of_successors_for_edit_session(
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        entity_id: String,
        change_set_id: String,
        edit_session_id: String,
    ) -> EntityResult<()> {
        let entity_id = entity_id.into();
        let change_set_id = change_set_id.into();
        let edit_session_id = edit_session_id.into();
        let pg = pg.clone();
        let nats_conn = nats_conn.clone();
        let veritech = veritech.clone();
        tokio::spawn(async move {
            match Entity::task_calculate_properties_of_successors_for_edit_session(
                pg,
                nats_conn,
                veritech,
                entity_id,
                change_set_id,
                edit_session_id,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    dbg!("failed to calculate properties of successors: {:?}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn get_head_by_name_and_entity_type(
        txn: &PgTxn<'_>,
        name: impl AsRef<str>,
        entity_type: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> EntityResult<Vec<Entity>> {
        let name = name.as_ref();
        let entity_type = entity_type.as_ref();
        let workspace_id = workspace_id.as_ref();

        let mut results = Vec::new();
        let rows = txn
            .query(
                ENTITY_GET_HEAD_BY_NAME_AND_ENTITY_TYPE,
                &[&name, &entity_type, &workspace_id],
            )
            .await?;
        for row in rows.into_iter() {
            let entity_json: serde_json::Value = row.try_get("object")?;
            let entity: Entity = serde_json::from_value(entity_json)?;
            results.push(entity);
        }
        Ok(results)
    }

    //pub async fn as_automerge(&self) -> EntityResult<String> {
    //    let reqwest = reqwest::Client::new();
    //    let res = reqwest
    //        .post("http://localhost:5157/wrapAutomerge")
    //        .json(&serde_json::json!(&self))
    //        .send()
    //        .await?;
    //    let results: serde_json::Value = res.json().await?;
    //    let doc = results["doc"]
    //        .as_str()
    //        .ok_or(EntityError::AutomergeVeritech)?;
    //    Ok(String::from(doc))
    //}
    //
    //
}
