use content_store::Store;
use petgraph::Direction;

use crate::change_set_pointer::ChangeSetPointer;
use crate::func::intrinsics::IntrinsicFunc;
use crate::provider::internal::{
    InternalProviderContent, InternalProviderContentV1, InternalProviderGraphNode,
};
use crate::socket::{DiagramKind, SocketEdgeKind, SocketKind};
use crate::workspace_snapshot::api::socket::SocketParent;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{NodeWeight, PropNodeWeight};
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{DalContext, FuncId, SchemaVariantId, SocketArity, Timestamp, WorkspaceSnapshot};

impl WorkspaceSnapshot {
    pub async fn internal_provider_create_implicit(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        prop: &PropNodeWeight,
    ) -> WorkspaceSnapshotResult<()> {
        for edgeref in self.edges_directed(prop.id(), Direction::Outgoing)? {
            if edgeref.weight().kind() == &EdgeWeightKind::Provider {
                // It already exists!
                return Ok(());
            }
        }

        let content = InternalProviderContentV1 {
            timestamp: Timestamp::now(),
            name: prop.name().to_string(),
            inbound_type_definition: None,
            outbound_type_definition: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&InternalProviderContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::InternalProvider(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let prop_node_index = self.working_copy()?.get_node_index_by_id(prop.id())?;
        self.working_copy()?.add_edge(
            prop_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Provider)?,
            node_index,
        )?;

        let func_id = self.func_find_intrinsic(IntrinsicFunc::Identity)?;
        let (_, _) = self
            .attribute_prototype_create(ctx, change_set, func_id)
            .await?;

        Ok(())
    }

    pub async fn internal_provider_create_explicit_with_socket(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        schema_variant_id: SchemaVariantId,
        name: impl AsRef<str>,
        func_id: FuncId,
        arity: SocketArity,
        frame_socket: bool,
    ) -> WorkspaceSnapshotResult<InternalProviderGraphNode> {
        let name = name.as_ref().to_string();
        let timestamp = Timestamp::now();

        let content = InternalProviderContentV1 {
            timestamp,
            name: name.clone(),
            inbound_type_definition: None,
            outbound_type_definition: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&InternalProviderContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::InternalProvider(hash))?;
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
                SocketEdgeKind::ConfigurationInput,
                arity,
                DiagramKind::Configuration,
                Some(schema_variant_id),
                SocketParent::ExplicitInternalProvider(id.into()),
            )
            .await?;

        Ok(InternalProviderGraphNode::assemble(id, hash, content))
    }
}
