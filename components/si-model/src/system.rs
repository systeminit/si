use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use thiserror::Error;

use crate::{
    Edge, EdgeError, Entity, EntityError, LabelList, LabelListItem, Node, NodeError, Veritech,
    Vertex,
};

const SYSTEM_LIST_AS_LABELS: &str = include_str!("./queries/system_list_as_labels.sql");

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("system name not found: {0}")]
    NameNotFound(String),
}

pub type SystemResult<T> = Result<T, SystemError>;

pub async fn list_as_labels(
    txn: &PgTxn<'_>,
    workspace_id: impl AsRef<str>,
) -> SystemResult<LabelList> {
    let workspace_id = workspace_id.as_ref();
    let mut results = Vec::new();
    let rows = txn.query(SYSTEM_LIST_AS_LABELS, &[&workspace_id]).await?;
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("item")?;
        let object: LabelListItem = serde_json::from_value(json)?;
        results.push(object);
    }

    return Ok(results);
}

pub async fn assign_entity_to_system_by_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    system_name: impl Into<String>,
    entity: &Entity,
) -> SystemResult<()> {
    let system_name = system_name.into();
    let mut systems = Entity::get_head_by_name_and_entity_type(
        &txn,
        &system_name,
        "system",
        &entity.si_storable.workspace_id,
    )
    .await?;
    if let Some(system) = systems.pop() {
        Edge::new(
            &txn,
            &nats,
            Vertex::from_entity(&system, "output"),
            Vertex::from_entity(&entity, "input"),
            false,
            crate::EdgeKind::Includes,
            &entity.si_storable.workspace_id,
        )
        .await?;
        Ok(())
    } else {
        Err(SystemError::NameNotFound(system_name))
    }
}

pub async fn create(
    pg: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    name: Option<String>,
    workspace_id: impl AsRef<str>,
    change_set_id: impl AsRef<str>,
    edit_session_id: impl AsRef<str>,
) -> SystemResult<Entity> {
    let workspace_id = workspace_id.as_ref();
    let change_set_id = change_set_id.as_ref();
    let edit_session_id = edit_session_id.as_ref();
    let node = Node::new(
        pg,
        txn,
        nats_conn,
        nats,
        veritech,
        name,
        "system",
        &workspace_id,
        &change_set_id,
        &edit_session_id,
    )
    .await?;
    let entity =
        Entity::for_edit_session(&txn, node.object_id, &change_set_id, &edit_session_id).await?;
    Ok(entity)
}
