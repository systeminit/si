use telemetry::prelude::*;

use super::SnapshotGraphMigratorResult;
use crate::{
    DalContext,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        graph::{
            WorkspaceSnapshotGraphV2,
            WorkspaceSnapshotGraphV3,
        },
        node_weight::{
            InputSocketNodeWeight,
            NodeWeight,
            SchemaVariantNodeWeight,
        },
    },
};

#[instrument(skip_all)]
pub async fn migrate_v2_to_v3(
    ctx: &DalContext,
    v2_graph: WorkspaceSnapshotGraphV2,
) -> SnapshotGraphMigratorResult<WorkspaceSnapshotGraphV3> {
    let mut v3_graph = WorkspaceSnapshotGraphV3::new_from_parts(
        v2_graph.graph().clone(),
        v2_graph.node_index_by_id().clone(),
        v2_graph.node_indices_by_lineage_id().clone(),
        v2_graph.root(),
    );

    let mut node_ids_to_upgrade = Vec::new();
    for (node_weight, _) in v3_graph.nodes() {
        match node_weight {
            NodeWeight::Content(content)
                if content.content_address_discriminants()
                    == ContentAddressDiscriminants::InputSocket
                    || content.content_address_discriminants()
                        == ContentAddressDiscriminants::SchemaVariant =>
            {
                node_ids_to_upgrade.push(content.id());
            }
            _ => {}
        }
    }

    for node_id in node_ids_to_upgrade {
        let old_node_weight = v3_graph
            .get_node_weight(v3_graph.get_node_index_by_id(node_id)?)?
            .clone();
        match old_node_weight {
            NodeWeight::Content(content)
                if content.content_address_discriminants()
                    == ContentAddressDiscriminants::InputSocket =>
            {
                InputSocketNodeWeight::try_upgrade_from_content_node_weight(
                    ctx,
                    &mut v3_graph,
                    &content,
                )
                .await?;
            }
            NodeWeight::Content(content)
                if content.content_address_discriminants()
                    == ContentAddressDiscriminants::SchemaVariant =>
            {
                SchemaVariantNodeWeight::try_upgrade_from_content_node_weight(
                    ctx,
                    &mut v3_graph,
                    &content,
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(v3_graph)
}
