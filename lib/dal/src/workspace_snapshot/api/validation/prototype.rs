use content_store::Store;

use crate::change_set_pointer::ChangeSetPointer;
use crate::func::backend::validation::FuncBackendValidationArgs;
use crate::func::intrinsics::IntrinsicFunc;
use crate::validation::prototype::{
    ValidationPrototypeContent, ValidationPrototypeContentV1, ValidationPrototypeGraphNode,
};
use crate::validation::Validation;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};

use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{DalContext, FuncId, PropId, Timestamp, WorkspaceSnapshot};

impl WorkspaceSnapshot {
    pub async fn validation_prototype_create_in_memory(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        validation: Validation,
        parent_prop_id: PropId,
    ) -> WorkspaceSnapshotResult<ValidationPrototypeGraphNode> {
        let func_id: FuncId = self.func_find_intrinsic(IntrinsicFunc::Validation)?;
        let args = serde_json::to_value(FuncBackendValidationArgs::new(validation))?;
        let validation_prototype_graph_node = self
            .validation_prototype_create(ctx, change_set, func_id, args, parent_prop_id)
            .await?;
        Ok(validation_prototype_graph_node)
    }

    pub async fn validation_prototype_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        func_id: FuncId,
        args: serde_json::Value,
        parent_prop_id: PropId,
    ) -> WorkspaceSnapshotResult<ValidationPrototypeGraphNode> {
        let timestamp = Timestamp::now();

        let content = ValidationPrototypeContentV1 {
            timestamp,
            func_id,
            args,
            link: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ValidationPrototypeContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ValidationPrototype(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let parent_prop_index = self
            .working_copy()?
            .get_node_index_by_id(parent_prop_id.into())?;
        self.working_copy()?.add_edge(
            parent_prop_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        Ok(ValidationPrototypeGraphNode::assemble(id, hash, content))
    }
}
