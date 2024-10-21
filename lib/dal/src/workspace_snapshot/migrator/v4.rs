use std::sync::Arc;
use telemetry::prelude::*;

use super::SnapshotGraphMigratorResult;
use crate::layer_db_types::{ViewContent, ViewContentV1};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::ComponentNodeWeight;
use crate::{
    workspace_snapshot::{
        graph::{WorkspaceSnapshotGraphV3, WorkspaceSnapshotGraphV4},
        node_weight::NodeWeight,
    },
    DalContext, EdgeWeight, EdgeWeightKind, Timestamp,
};

#[instrument(skip_all)]
pub async fn migrate_v3_to_v4(
    ctx: &DalContext,
    v3_graph: WorkspaceSnapshotGraphV3,
) -> SnapshotGraphMigratorResult<WorkspaceSnapshotGraphV4> {
    let mut v4_graph = WorkspaceSnapshotGraphV4::new_from_parts(
        v3_graph.graph().clone(),
        v3_graph.node_index_by_id().clone(),
        v3_graph.node_indices_by_lineage_id().clone(),
        v3_graph.root(),
    );

    // Create new category nodes
    {
        let id = v4_graph.generate_ulid()?;
        let lineage_id = v4_graph.generate_ulid()?;
        let category_node_index =
            v4_graph.add_category_node(id, lineage_id, CategoryNodeKind::DiagramObject)?;
        v4_graph.add_edge(
            v4_graph.root(),
            EdgeWeight::new(EdgeWeightKind::new_use()),
            category_node_index,
        )?;
    }

    let category_node_index = {
        let id = v4_graph.generate_ulid()?;
        let lineage_id = v4_graph.generate_ulid()?;
        let category_node_index =
            v4_graph.add_category_node(id, lineage_id, CategoryNodeKind::View)?;
        v4_graph.add_edge(
            v4_graph.root(),
            EdgeWeight::new(EdgeWeightKind::new_use()),
            category_node_index,
        )?;

        category_node_index
    };

    // Create default view
    let default_view_idx = {
        let id = v4_graph.generate_ulid()?;
        let lineage_id = v4_graph.generate_ulid()?;

        let content = ViewContent::V1(ViewContentV1 {
            timestamp: Timestamp::now(),
            name: "DEFAULT".to_owned(),
        });

        let (content_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(content.clone().into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let node_weight = NodeWeight::new_view(id, lineage_id, content_address);
        let default_view_node_idx = v4_graph.add_or_replace_node(node_weight.clone())?;

        v4_graph.add_edge(
            category_node_index,
            EdgeWeight::new(EdgeWeightKind::new_use_default()),
            default_view_node_idx,
        )?;

        default_view_node_idx
    };

    // Gather component nodes to upgrade
    let mut node_ids_to_upgrade = Vec::new();
    for (node_weight, _) in v4_graph.nodes() {
        if let NodeWeight::Component(content) = node_weight {
            node_ids_to_upgrade.push(content.id());
        }
    }

    for node_id in node_ids_to_upgrade {
        let old_node_weight = v4_graph
            .get_node_weight(v4_graph.get_node_index_by_id(node_id)?)?
            .clone();
        if let NodeWeight::Component(content) = old_node_weight {
            ComponentNodeWeight::try_upgrade_and_create_external_geometry(
                ctx,
                &mut v4_graph,
                default_view_idx,
                &content,
            )
            .await?;
        }
    }

    Ok(v4_graph)
}
