use content_store::{Store};
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointer;



use crate::socket::{SocketContent, SocketContentV1, SocketEdgeKind, SocketGraphNode, SocketKind, DiagramKind};

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{
    DalContext, ExternalProviderId, InternalProviderId,
    SchemaVariantId, SocketArity, Timestamp, WorkspaceSnapshot,
};

pub enum SocketParent {
    ExplicitInternalProvider(InternalProviderId),
    ExternalProvider(ExternalProviderId),
}

impl WorkspaceSnapshot {
    pub async fn socket_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        kind: SocketKind,
        socket_edge_kind: SocketEdgeKind,
        arity: SocketArity,
        diagram_kind: DiagramKind,
        _schema_variant_id: Option<SchemaVariantId>,
        socket_parent: SocketParent,
    ) -> WorkspaceSnapshotResult<SocketGraphNode> {
        let name = name.as_ref();
        let timestamp = Timestamp::now();

        let content = SocketContentV1 {
            timestamp,
            name: name.to_string(),
            human_name: None,
            kind,
            edge_kind: socket_edge_kind,
            diagram_kind,
            arity,
            required: false,
            ui_hidden: false,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&SocketContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Socket(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let parent_id: Ulid = match socket_parent {
            SocketParent::ExplicitInternalProvider(explicit_internal_provider_id) => {
                explicit_internal_provider_id.into()
            }
            SocketParent::ExternalProvider(external_provider_id) => external_provider_id.into(),
        };

        let parent_node_index = self.working_copy()?.get_node_index_by_id(parent_id)?;
        self.working_copy()?.add_edge(
            parent_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        Ok(SocketGraphNode::assemble(id, hash, content))
    }
}
