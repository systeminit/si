use async_trait::async_trait;
use si_id::{
    AttributeValueId,
    ComponentId,
    InputSocketId,
};

use crate::{
    DalContext,
    InputSocket,
    SchemaVariantId,
    WorkspaceSnapshot,
    WorkspaceSnapshotError,
    layer_db_types::{
        InputSocketContent,
        InputSocketContentV2,
    },
    socket::input::InputSocketResult,
    workspace_snapshot::{
        WorkspaceSnapshotResult,
        graph::InputSocketExt as InputSocketExtGraph,
        node_weight::{
            InputSocketNodeWeight,
            traits::SiVersionedNodeWeight,
        },
    },
};

#[async_trait]
pub trait InputSocketExt {
    /// Retrieve the [`InputSocket`] with the specified [`InputSocketId`].
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket>;

    /// Retrieve the [`InputSocket`] with the specified name for the given [`SchemaVariantId`].
    async fn get_input_socket_by_name_opt(
        &self,
        ctx: &DalContext,
        name: &str,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Option<InputSocket>>;

    /// List all [`InputSocketId`]s for the given [`SchemaVariantId`].
    async fn list_input_socket_ids_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocketId>>;

    /// List all [`InputSocket`] for the given [`SchemaVariantId`].
    async fn list_input_sockets(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocket>>;

    /// Get all [`AttributeValueId`] across all [`Component`][crate::Component] for the given
    /// [`InputSocketId`]
    ///
    /// NOTE: call component_attribute_value_for_input_socket_id() if you want the attribute
    /// value for a specific component.
    ///
    async fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<Vec<AttributeValueId>>;

    async fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<AttributeValueId>;

    async fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<InputSocketId>>;
}

#[async_trait]
impl InputSocketExt for WorkspaceSnapshot {
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket> {
        let input_socket_node_weight = self.working_copy().await.get_input_socket(id)?;

        input_socket_from_node_weight(ctx, &input_socket_node_weight)
            .await
            .map_err(Box::new)
            .map_err(Into::into)
    }

    async fn get_input_socket_by_name_opt(
        &self,
        ctx: &DalContext,
        name: &str,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Option<InputSocket>> {
        Ok(self
            .list_input_sockets(ctx, schema_variant_id)
            .await?
            .iter()
            .find(|input_socket| input_socket.name() == name)
            .cloned())
    }

    async fn list_input_socket_ids_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocketId>> {
        Ok(self
            .working_copy()
            .await
            .list_input_sockets_for_schema_variant(schema_variant_id)?
            .iter()
            .map(|input_node_weight| input_node_weight.id().into())
            .collect())
    }

    async fn list_input_sockets(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocket>> {
        let mut result = Vec::new();
        for input_socket_node_weight in self
            .working_copy()
            .await
            .list_input_sockets_for_schema_variant(schema_variant_id)?
        {
            let input_socket = input_socket_from_node_weight(ctx, &input_socket_node_weight)
                .await
                .map_err(Box::new)
                .map_err(WorkspaceSnapshotError::from)?;
            result.push(input_socket);
        }

        Ok(result)
    }

    async fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<Vec<AttributeValueId>> {
        self.working_copy()
            .await
            .all_attribute_value_ids_everywhere_for_input_socket_id(input_socket_id)
            .map_err(Into::into)
    }

    async fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<AttributeValueId> {
        self.working_copy()
            .await
            .component_attribute_value_id_for_input_socket_id(input_socket_id, component_id)
            .map_err(Into::into)
    }

    async fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<InputSocketId>> {
        self.working_copy()
            .await
            .input_socket_id_find_for_attribute_value_id(attribute_value_id)
            .map_err(Into::into)
    }
}

pub(crate) async fn input_socket_from_node_weight(
    ctx: &DalContext,
    input_socket_node_weight: &InputSocketNodeWeight,
) -> InputSocketResult<InputSocket> {
    let content: InputSocketContent = ctx
        .layer_db()
        .cas()
        .try_read_as(&input_socket_node_weight.content_hash())
        .await?
        .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
            input_socket_node_weight.id(),
        ))?;

    input_socket_from_node_weight_and_content(input_socket_node_weight, content)
}

#[inline(always)]
pub(crate) fn input_socket_from_node_weight_and_content(
    node_weight: &InputSocketNodeWeight,
    content: InputSocketContent,
) -> InputSocketResult<InputSocket> {
    let input_socket = match content {
        InputSocketContent::V1(v1_inner) => {
            let v2_inner = InputSocketContentV2 {
                timestamp: v1_inner.timestamp,
                name: v1_inner.name.clone(),
                inbound_type_definition: v1_inner.inbound_type_definition.clone(),
                outbound_type_definition: v1_inner.outbound_type_definition.clone(),
                kind: v1_inner.kind,
                required: v1_inner.required,
                ui_hidden: v1_inner.ui_hidden,
                connection_annotations: v1_inner.connection_annotations.clone(),
            };

            InputSocket::assemble(node_weight.id().into(), v1_inner.arity, v2_inner)
        }
        InputSocketContent::V2(inner) => InputSocket::assemble(
            node_weight.id().into(),
            node_weight.inner().arity(),
            inner.clone(),
        ),
    };

    Ok(input_socket)
}
