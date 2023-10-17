use content_store::{ContentHash, Store};
use petgraph::visit::Time;

use crate::change_set_pointer::ChangeSetPointer;
use crate::component::ComponentKind;
use crate::component::{ComponentContent, ComponentContentV1, ComponentGraphNode};
use crate::node::NodeContentV1;
use crate::node::NodeKind;
use crate::node::{NodeContent, NodeGraphNode};
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::ComponentId;
use crate::{Component, DalContext, Node, NodeId, SchemaVariantId, Timestamp, WorkspaceSnapshot};

impl WorkspaceSnapshot {
    pub async fn node_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        kind: Option<NodeKind>,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<ComponentGraphNode> {
        let content = NodeContentV1 {
            timestamp: Timestamp::now(),
            kind: match kind {
                Some(provided_kind) => provided_kind,
                None => NodeKind::Configuration,
            },
            ..Default::default()
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&NodeContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(&change_set, id, ContentAddress::Node(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        // Component --> Node (this)
        let component_index = self.get_node_index_by_id(component_id)?;
        self.working_copy()?.add_edge(
            component_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        Ok(NodeGraphNode::assemble(id, hash, content))
    }

    async fn node_get_content(
        &mut self,
        ctx: &DalContext,
        node_id: NodeId,
    ) -> WorkspaceSnapshotResult<(ContentHash, NodeContentV1)> {
        let id: Ulid = node_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: NodeContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let inner = match content {
            NodeContentV1::V1(inner) => inner,
        };

        Ok((hash, inner))
    }

    pub async fn node_set_geometry(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        node_id: NodeId,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
        width: Option<impl AsRef<str>>,
        height: Option<impl AsRef<str>>,
    ) -> WorkspaceSnapshotResult<()> {
        let (_, inner) = self.node_get_content(ctx, node_id).await?;

        let mut node = Node::assemble(node_id, &inner);
        node.x = x;
        node.y = y;
        node.width = width;
        node.height = height;
        let updated = NodeContentV1::from(node);

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&NodeContent::V1(updated.clone()))?;

        self.working_copy()?
            .update_content(&change_set, node_id.into(), hash)?;

        Ok(())
    }
}
