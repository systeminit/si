use content_store::Store;

use crate::change_set_pointer::ChangeSetPointer;
use crate::provider::external::{
    ExternalProviderContent, ExternalProviderContentV1, ExternalProviderGraphNode,
};

use crate::socket::{DiagramKind, SocketEdgeKind, SocketKind};

use crate::workspace_snapshot::api::socket::SocketParent;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{DalContext, FuncId, SchemaVariantId, SocketArity, Timestamp, WorkspaceSnapshot};

impl WorkspaceSnapshot {
    pub async fn external_provider_create_with_socket(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        schema_variant_id: SchemaVariantId,
        name: impl AsRef<str>,
        type_definition: Option<String>,
        func_id: FuncId,
        arity: SocketArity,
        frame_socket: bool,
    ) -> WorkspaceSnapshotResult<ExternalProviderGraphNode> {
        let name = name.as_ref();
        let timestamp = Timestamp::now();

        let content = ExternalProviderContentV1 {
            timestamp,
            schema_variant_id,
            attribute_prototype_id: None,
            name: name.to_string(),
            type_definition,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ExternalProviderContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ExternalProvider(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let schema_variant_node_index = self
            .working_copy()?
            .get_node_index_by_id(schema_variant_id.into())?;
        self.working_copy()?.add_edge(
            schema_variant_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Provider)?,
            node_index,
        )?;

        let _attribute_prototype = self
            .attribute_prototype_create(ctx, change_set, func_id)
            .await?;

        let _socket = self
            .socket_create(
                ctx,
                change_set,
                name,
                match frame_socket {
                    true => SocketKind::Frame,
                    false => SocketKind::Provider,
                },
                SocketEdgeKind::ConfigurationOutput,
                arity,
                DiagramKind::Configuration,
                Some(schema_variant_id),
                SocketParent::ExternalProvider(id.into()),
            )
            .await?;

        Ok(ExternalProviderGraphNode::assemble(id, hash, content))
    }
}
