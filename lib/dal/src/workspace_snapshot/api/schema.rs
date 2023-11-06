use content_store::{ContentHash, Store};
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointer;
use crate::schema::{ComponentKind, SchemaContent, SchemaContentV1, SchemaGraphNode};
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::{DalContext, SchemaId, Timestamp, WorkspaceSnapshot};

pub mod variant;

impl WorkspaceSnapshot {
    pub async fn schema_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        component_kind: ComponentKind,
    ) -> WorkspaceSnapshotResult<SchemaGraphNode> {
        let name = name.as_ref();
        let timestamp = Timestamp::now();
        let ui_hidden = false;

        let content = SchemaContentV1 {
            timestamp,
            name: name.to_string(),
            ui_hidden,
            default_schema_variant_id: None,
            component_kind,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&SchemaContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Schema(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let schema_category_index = self
            .working_copy()?
            .get_category(CategoryNodeKind::Schema)?;
        /*self.working_copy()?.add_edge(
        schema_category_index,
        EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
        node_index,
        )?;*/

        Ok(SchemaGraphNode::assemble(id, hash, content))
    }

    pub async fn schema_get_content(
        &mut self,
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> WorkspaceSnapshotResult<(ContentHash, SchemaContentV1)> {
        let id: Ulid = schema_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: SchemaContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let inner = match content {
            SchemaContent::V1(inner) => inner,
        };

        Ok((hash, inner))
    }
}
