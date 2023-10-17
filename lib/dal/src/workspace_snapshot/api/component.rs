use content_store::{ContentHash, Store};

use crate::change_set_pointer::ChangeSetPointer;
use crate::component::ComponentKind;
use crate::component::{ComponentContent, ComponentContentV1, ComponentGraphNode};
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{Component, DalContext, NodeKind, SchemaVariantId, Timestamp, WorkspaceSnapshot};

impl WorkspaceSnapshot {
    pub async fn component_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
        component_kind: Option<ComponentKind>,
    ) -> WorkspaceSnapshotResult<ComponentGraphNode> {
        let name = name.as_ref();
        let timestamp = Timestamp::now();
        let ui_hidden = false;

        let content = ComponentContentV1 {
            timestamp,
            kind: match component_kind {
                Some(provided_kind) => provided_kind,
                None => ComponentKind::Standard,
            },
            needs_destroy: false,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ComponentContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(&change_set, id, ContentAddress::Component(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        // Root --> Component Category --> Component (this)
        let component_category_index = self
            .working_copy()?
            .get_category_child(CategoryNodeKind::Component)?;
        self.working_copy()?.add_edge(
            component_category_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        // Component (this) --> Schema Variant
        let schema_variant_index = self.get_node_index_by_id(schema_variant_id.into())?;
        self.working_copy()?.add_edge(
            node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            schema_variant_index,
        )?;

        // Create a node. When a node is created an edge will be created from the component node index (this) to the new "node node". Totally not confusing...
        self.node_create(ctx, change_set, Some(NodeKind::Configuration), id.into())
            .await?;

        Ok(ComponentGraphNode::assemble(id, hash, content))
    }
}
