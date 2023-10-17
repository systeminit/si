use content_store::{Store};
use petgraph::prelude::*;


use crate::attribute::prototype::{
    AttributePrototypeContent, AttributePrototypeContentV1,
};
use crate::change_set_pointer::ChangeSetPointer;





use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::{
    AttributePrototype, AttributePrototypeId, DalContext, FuncId, Timestamp,
    WorkspaceSnapshot,
};

impl WorkspaceSnapshot {
    // NOTE(nick,jacob,zack): all incoming edges to an attribute prototype must come from one of two places:
    //   - an attribute value whose lineage comes from a component
    //   - a prop whose lineage comes from a schema variant
    // Outgoing edges from an attribute prototype are used for intra and inter component relationships.
    pub async fn attribute_prototype_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        func_id: FuncId,
    ) -> WorkspaceSnapshotResult<(AttributePrototype, NodeIndex)> {
        let timestamp = Timestamp::now();

        let content = AttributePrototypeContentV1 { timestamp };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&AttributePrototypeContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributePrototype(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let func_node_index = self.working_copy()?.get_node_index_by_id(func_id.into())?;
        self.working_copy()?.add_edge(
            node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_node_index,
        )?;

        Ok((
            AttributePrototype::assemble(AttributePrototypeId::from(id), &content),
            node_index,
        ))
    }

    pub fn attribute_prototype_update_func(
        &mut self,
        change_set: &ChangeSetPointer,
        attribute_prototype_id: AttributePrototypeId,
        func_id: FuncId,
    ) -> WorkspaceSnapshotResult<()> {
        let attribute_prototype_idx = self
            .working_copy()?
            .get_node_index_by_id(attribute_prototype_id.into())?;

        let current_func_node_idx = self
            .edges_directed(attribute_prototype_id.into(), Direction::Outgoing)?
            .find(|edge_ref| edge_ref.weight().kind() == &EdgeWeightKind::Use)
            .map(|edge_ref| edge_ref.target())
            .ok_or(WorkspaceSnapshotError::AttributePrototypeMissingFunction(
                attribute_prototype_id,
            ))?;

        self.working_copy()?.remove_edge(
            change_set,
            attribute_prototype_idx,
            current_func_node_idx,
            EdgeWeightKindDiscriminants::Use,
        )?;

        // Node index changes after edge removal, so we have to fetch it again
        let attribute_prototype_idx = self
            .working_copy()?
            .get_node_index_by_id(attribute_prototype_id.into())?;

        let func_node_idx = self.working_copy()?.get_node_index_by_id(func_id.into())?;

        self.working_copy()?.add_edge(
            attribute_prototype_idx,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_node_idx,
        )?;

        Ok(())
    }
}
