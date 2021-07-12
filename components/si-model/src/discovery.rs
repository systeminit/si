use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::{NatsConn, NatsTxnError, PgPool, PgTxn};

use crate::{
    entity::Op,
    resource,
    workflow::selector::{SelectionEntry, SelectionEntryPredecessor},
    ChangeSet, ChangeSetError, Edge, EdgeError, EdgeKind, EditSession, EditSessionError, Entity,
    EntityError, Node, NodeError, NodePosition, NodePositionError, Resource, ResourceError,
    Veritech, VeritechError, Vertex, WorkflowError,
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
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("no conceptual node could be found")]
    NoConcept,
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
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

pub async fn import_concept(
    pg: &PgPool,
    nats_conn: &NatsConn,
    _veritech: &Veritech,
    workspace_id: impl AsRef<str>,
    application_id: impl AsRef<str>,
    implementation_entity_id: impl AsRef<str>,
) -> DiscoveryResult<()> {
    let workspace_id = workspace_id.as_ref();
    let application_id = application_id.as_ref();
    let implementation_entity_id = implementation_entity_id.as_ref();

    let mut conn = pg.pool.get().await?;
    let txn = conn.transaction().await?;
    let nats = nats_conn.transaction();

    let application_entity = Entity::for_head(&txn, &application_id).await?;

    let mut concept_edges =
        Edge::by_kind_and_head_object_id(&txn, &EdgeKind::Component, &implementation_entity_id)
            .await?;
    let concept_edge = concept_edges.pop().ok_or(DiscoveryError::NoConcept)?;
    let concept_entity = Entity::for_head(&txn, &concept_edge.tail_vertex.object_id).await?;

    // Add an edge for the concept to the application
    let _edge = Edge::new(
        &txn,
        &nats,
        Vertex::from_entity(&application_entity, "output"),
        Vertex::from_entity(&concept_entity, "application"),
        false,
        EdgeKind::Component,
        workspace_id.clone(),
    )
    .await?;

    match Edge::new(
        &txn,
        &nats,
        Vertex::from_entity(&application_entity, "output"),
        Vertex::from_entity(&concept_entity, "includes"),
        false,
        EdgeKind::Includes,
        workspace_id.clone(),
    )
    .await
    {
        Ok(_edge) => {}
        Err(EdgeError::EdgeExists) => {}
        Err(e) => return Err(DiscoveryError::from(e)),
    };

    // Set position on the deployment diagram
    let pos_deployment_context_id = format!("{}.deployment", application_entity.id);
    let pos_context_id = format!("{}.component", concept_entity.id);
    let concept_deployment_x: f64 = 3893.0;
    let mut concept_deployment_y: f64 = 4131.0;
    if concept_entity.entity_type == "service" {
        concept_deployment_y = concept_deployment_y - 125 as f64;
    }
    let concept_component_x: f64 = 3893.0;
    let concept_component_y: f64 = 4131.0;

    let _concept_deployment_node_positions = NodePosition::create_or_update(
        &txn,
        &nats,
        &concept_entity.node_id,
        &pos_deployment_context_id,
        format!("{:.0}", concept_deployment_x),
        format!("{:.0}", concept_deployment_y),
        &workspace_id,
    )
    .await?;
    let _concept_component_node_positions = NodePosition::create_or_update(
        &txn,
        &nats,
        &concept_entity.node_id,
        &pos_context_id,
        format!("{:.0}", concept_component_x),
        format!("{:.0}", concept_component_y),
        &workspace_id,
    )
    .await?;

    let impl_entity = Entity::for_head(&txn, &implementation_entity_id).await?;
    let impl_entity_predecessor_edges =
        Edge::all_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &impl_entity.id)
            .await?;
    let mut impls_to_import = vec![impl_entity];
    for edge in impl_entity_predecessor_edges.into_iter() {
        let sentity = Entity::for_head(&txn, &edge.tail_vertex.object_id).await?;
        impls_to_import.push(sentity);
    }

    let mut impl_pos_x: f64 = concept_component_x.clone();
    let impl_pos_y: f64 = concept_component_y.clone();

    let mut seen_ids: Vec<String> = vec![];
    for i_entity in impls_to_import.iter() {
        let has_id = seen_ids
            .iter()
            .find(|id| &i_entity.id[..] == &id[..])
            .is_some();
        if has_id {
            continue;
        } else {
            seen_ids.push(i_entity.id.clone());
        }

        impl_pos_x = impl_pos_x - 200 as f64;
        NodePosition::create_or_update(
            &txn,
            &nats,
            &i_entity.node_id,
            &pos_context_id,
            format!("{:.0}", impl_pos_x),
            format!("{:.0}", impl_pos_y),
            &workspace_id,
        )
        .await?;

        match Edge::new(
            &txn,
            &nats,
            Vertex::from_entity(&application_entity, "output"),
            Vertex::from_entity(&i_entity, "includes"),
            false,
            EdgeKind::Includes,
            workspace_id.clone(),
        )
        .await
        {
            Ok(_edge) => {}
            Err(EdgeError::EdgeExists) => {}
            Err(e) => return Err(DiscoveryError::from(e)),
        };
    }

    match concept_entity.entity_type.as_ref() {
        "service" => {
            let all_entities_for_app = Edge::direct_successor_edges_by_object_id(
                &txn,
                &EdgeKind::Component,
                &application_entity.id,
            )
            .await?;
            let kubernetes_cluster_edge = all_entities_for_app
                .into_iter()
                .find(|e| e.head_vertex.object_type == "kubernetesCluster");
            if let Some(kubernetes_cluster_edge) = kubernetes_cluster_edge {
                let kubernetes_cluster_entity =
                    Entity::for_head(&txn, kubernetes_cluster_edge.head_vertex.object_id).await?;
                let _edge = Edge::new(
                    &txn,
                    &nats,
                    Vertex::from_entity(&concept_entity, "output"),
                    Vertex::from_entity(&kubernetes_cluster_entity, &concept_entity.entity_type),
                    false,
                    EdgeKind::Deployment,
                    workspace_id.clone(),
                )
                .await?;
            }
        }
        "kubernetesCluster" => {
            let all_entities_for_app = Edge::direct_successor_edges_by_object_id(
                &txn,
                &EdgeKind::Component,
                &application_entity.id,
            )
            .await?;
            let cloud_edge = all_entities_for_app
                .into_iter()
                .find(|e| e.head_vertex.object_type == "cloudProvider");
            if let Some(cloud_edge) = cloud_edge {
                let cloud_entity = Entity::for_head(&txn, cloud_edge.head_vertex.object_id).await?;
                let _edge = Edge::new(
                    &txn,
                    &nats,
                    Vertex::from_entity(&concept_entity, "output"),
                    Vertex::from_entity(&cloud_entity, &concept_entity.entity_type),
                    false,
                    EdgeKind::Deployment,
                    workspace_id.clone(),
                )
                .await?;
            }
        }
        _ => {}
    }

    txn.commit().await?;

    Ok(())
}

pub async fn import_implementation(
    pg: &PgPool,
    nats_conn: &NatsConn,
    _veritech: &Veritech,
    workspace_id: impl AsRef<str>,
    application_id: impl AsRef<str>,
    importing_entity_id: impl AsRef<str>,
    implementation_entity_id: impl AsRef<str>,
) -> DiscoveryResult<()> {
    let workspace_id = workspace_id.as_ref();
    let application_id = application_id.as_ref();
    let importing_entity_id = importing_entity_id.as_ref();
    let implementation_entity_id = implementation_entity_id.as_ref();

    let mut conn = pg.pool.get().await?;
    let txn = conn.transaction().await?;
    let nats = nats_conn.transaction();

    let application_entity = Entity::for_head(&txn, &application_id).await?;
    let importing_concept_entity = Entity::for_head(&txn, &importing_entity_id).await?;
    let importing_concept_node_positions =
        NodePosition::get_by_node_id(&txn, &importing_concept_entity.node_id).await?;
    let impl_entity = Entity::for_head(&txn, &implementation_entity_id).await?;
    let impl_entity_predecessor_edges =
        Edge::all_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &impl_entity.id)
            .await?;
    let mut impls_to_import = vec![impl_entity];
    for edge in impl_entity_predecessor_edges.into_iter() {
        let sentity = Entity::for_head(&txn, &edge.tail_vertex.object_id).await?;
        impls_to_import.push(sentity);
    }

    let pos_context_id = format!("{}.component", importing_concept_entity.id);
    let mut impl_pos_x: f64 = 0 as f64;
    let mut impl_pos_y: f64 = 0 as f64;

    if let Some(concept_pos) = importing_concept_node_positions
        .iter()
        .find(|cnp| cnp.context_id == pos_context_id)
    {
        let concept_pos_x: f64 = concept_pos.x.parse().expect("should be a number");
        let concept_pos_y: f64 = concept_pos.x.parse().expect("should be a number");
        impl_pos_x = concept_pos_x;
        impl_pos_y = concept_pos_y;
    }
    for i_entity in impls_to_import.iter() {
        impl_pos_x = impl_pos_x - 200 as f64;
        NodePosition::create_or_update(
            &txn,
            &nats,
            &i_entity.node_id,
            &pos_context_id,
            format!("{:.0}", impl_pos_x),
            format!("{:.0}", impl_pos_y),
            &workspace_id,
        )
        .await?;

        let _edge = Edge::new(
            &txn,
            &nats,
            Vertex::from_entity(&application_entity, "output"),
            Vertex::from_entity(&i_entity, "includes"),
            false,
            EdgeKind::Includes,
            workspace_id.clone(),
        )
        .await?;

        let _edge = Edge::new(
            &txn,
            &nats,
            Vertex::from_entity(&importing_concept_entity, "output"),
            Vertex::from_entity(&i_entity, "deployment"),
            false,
            EdgeKind::Component,
            workspace_id.clone(),
        )
        .await?;
    }
    txn.commit().await?;

    Ok(())
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
    array_meta: serde_json::Value,
    properties: serde_json::Value,
    code: serde_json::Value,
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

    let mut seen_entities: Vec<Entity> = vec![];
    veritech
        .send_async("discover", request, progress_tx)
        .await?;
    while let Some(message) = progress_rx.recv().await {
        match message {
            DiscoveryProtocol::Start(_) => {}
            DiscoveryProtocol::Finish(finish) => {
                for discovered in finish.discovered.into_iter() {
                    let mut concept_entity: Option<Entity> = None;
                    let mut namespace_entity: Option<Entity> = None;
                    let concept = discovered.clone();
                    let mut to_discover: Vec<(Option<Entity>, DiscoverEntity)> =
                        vec![(None, discovered)];
                    let mut index = 0;
                    while index < to_discover.len() {
                        let txn = conn.transaction().await?;
                        let nats = nats_conn.transaction();
                        let (parent, discover) = to_discover[index].clone();
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
                            entity.properties = discover.entity.properties.clone();
                            entity.code = discover.entity.code.clone();
                            entity.array_meta = discover.entity.array_meta.clone();
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

                            if entity.entity_type == "k8sNamespace" {
                                namespace_entity = Some(entity.clone());
                            }

                            // If this is the conceptual entity itself, replace the stub version
                            // with the fully realized one.
                            if concept.entity.entity_type == entity.entity_type {
                                concept_entity = Some(entity.clone());
                            }

                            if let Some(concept_entity) = concept_entity.as_mut() {
                                if concept_entity.id != entity.id {
                                    let _edge = Edge::new(
                                        &txn,
                                        &nats,
                                        Vertex::from_entity(&concept_entity, "output"),
                                        Vertex::from_node(&node, "deployment"),
                                        false,
                                        EdgeKind::Component,
                                        workspace_id.clone(),
                                    )
                                    .await?;
                                }
                                if index == 1 {
                                    concept_entity.ops.push(Op {
                                        op: crate::entity::OpType::Set,
                                        source: crate::entity::OpSource::Manual,
                                        system: system.id.clone(),
                                        path: vec!["implementation".to_string()],
                                        value: serde_json::json![entity.id],
                                        from: None,
                                    });
                                    concept_entity
                                        .infer_properties_for_edit_session(
                                            &txn,
                                            &veritech,
                                            &change_set.id,
                                            &edit_session.id,
                                        )
                                        .await?;
                                    concept_entity
                                        .save_for_edit_session(
                                            &txn,
                                            &change_set.id,
                                            &edit_session.id,
                                        )
                                        .await?;
                                    seen_entities.push(concept_entity.clone());
                                }
                            }

                            if let Some(parent_entity) = parent {
                                let tail_vertex = Vertex::new(
                                    &entity.node_id,
                                    &entity.id,
                                    "output",
                                    &entity.entity_type,
                                );
                                let head_vertex = match parent_entity.entity_type.as_ref() {
                                    // TODO: This is a big hack! We need to detect if we're a
                                    // conceptual entity or not from the schema, and make the right
                                    // decision.
                                    "kubernetesCluster" | "service" => Vertex::new(
                                        &parent_entity.node_id,
                                        &parent_entity.id,
                                        "implementations",
                                        &entity.entity_type,
                                    ),
                                    _ => Vertex::new(
                                        &parent_entity.node_id,
                                        &parent_entity.id,
                                        &entity.entity_type,
                                        &parent_entity.entity_type,
                                    ),
                                };
                                match Edge::new(
                                    &txn,
                                    &nats,
                                    tail_vertex,
                                    head_vertex,
                                    false,
                                    EdgeKind::Configures,
                                    &workspace_id,
                                )
                                .await
                                {
                                    Ok(_edge) => {}
                                    Err(EdgeError::EdgeExists) => {}
                                    Err(e) => return Err(DiscoveryError::from(e)),
                                };
                            }

                            edit_session.save_session(&txn).await?;
                            change_set.apply(&txn).await?;
                            if let Some(namespace_entity) = namespace_entity.as_ref() {
                                if entity.entity_type == "k8sDeployment" {
                                    match Edge::new(
                                        &txn,
                                        &nats,
                                        Vertex::from_entity(&namespace_entity, "output"),
                                        Vertex::from_entity(&entity, &namespace_entity.entity_type),
                                        false,
                                        EdgeKind::Configures,
                                        workspace_id.clone(),
                                    )
                                    .await
                                    {
                                        Ok(_edge) => {}
                                        Err(EdgeError::EdgeExists) => {}
                                        Err(e) => return Err(DiscoveryError::from(e)),
                                    };
                                }
                            }

                            txn.commit().await?;
                            nats.commit().await?;

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
                            index = index + 1;

                            if let Some(concept_entity) = concept_entity.as_ref() {
                                if concept_entity.id != entity.id {
                                    seen_entities.push(entity.clone());
                                }
                            } else {
                                seen_entities.push(entity.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    let txn = conn.transaction().await?;
    let nats = nats_conn.transaction();
    let mut change_set =
        ChangeSet::new(&txn, &nats, Some(format!("d prop",)), workspace_id.clone()).await?;
    let mut edit_session = EditSession::new(
        &txn,
        &nats,
        None,
        change_set.id.clone(),
        workspace_id.clone(),
    )
    .await?;

    for entity in seen_entities.iter_mut().rev() {
        entity
            .infer_properties_for_edit_session(&txn, &veritech, &change_set.id, &edit_session.id)
            .await?;
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
        entity
            .save_for_edit_session(&txn, &change_set.id, &edit_session.id)
            .await?;
    }
    edit_session.save_session(&txn).await?;
    change_set.apply(&txn).await?;
    txn.commit().await?;
    nats.commit().await?;

    Ok(())
}
