use std::time::Instant;

use content_store::{ContentHash, Store};
use std::collections::HashMap;

use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointer;
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::{FuncContent, FuncContentV1, FuncGraphNode};

use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{FuncNodeWeight, NodeWeight};
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::{
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncId, Timestamp,
    WorkspaceSnapshot,
};
use telemetry::prelude::*;

impl WorkspaceSnapshot {
    pub async fn func_create(
        &mut self,
        ctx: &DalContext,
        name: impl AsRef<str>,
        display_name: Option<impl AsRef<str>>,
        description: Option<impl AsRef<str>>,
        link: Option<impl AsRef<str>>,
        hidden: bool,
        builtin: bool,
        backend_kind: FuncBackendKind,
        backend_response_type: FuncBackendResponseType,
        handler: Option<impl AsRef<str>>,
        code_base64: Option<impl AsRef<str>>,
    ) -> WorkspaceSnapshotResult<Func> {
        let name = name.as_ref().to_string();
        let timestamp = Timestamp::now();
        let _finalized_once = false;

        let code_base64 = code_base64.map(|c| c.as_ref().to_string());

        let code_blake3 = ContentHash::new(code_base64.as_deref().unwrap_or("").as_bytes());

        let content = FuncContentV1 {
            timestamp,
            display_name: display_name.map(|d| d.as_ref().to_string()),
            description: description.map(|d| d.as_ref().to_string()),
            link: link.map(|l| l.as_ref().to_string()),
            hidden,
            builtin,
            backend_response_type,
            handler: handler.map(|h| h.as_ref().to_string()),
            code_base64,
            code_blake3,
        };

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&FuncContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_func(change_set, id, name.clone(), backend_kind, hash)?;
        let node_index = self.working_copy()?.add_node(node_weight.clone())?;

        let (_, func_category_index) = self.working_copy()?.get_category(CategoryNodeKind::Func)?;
        self.working_copy()?.add_edge(
            func_category_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        let func_node_weight = node_weight.get_func_node_weight()?;

        Ok(Func::assemble(&func_node_weight, &content))
    }

    pub async fn func_get_node_weight_and_content(
        &mut self,
        ctx: &DalContext,
        func_id: FuncId,
    ) -> WorkspaceSnapshotResult<(FuncNodeWeight, FuncContentV1)> {
        let id: Ulid = func_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: FuncContent = ctx
            .content_store()
            .try_lock()?
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let FuncContent::V1(inner) = content;

        let func_node_weight = node_weight.get_func_node_weight()?;

        Ok((func_node_weight, inner))
    }

    pub async fn func_get_by_id(
        &mut self,
        ctx: &DalContext,
        func_id: FuncId,
    ) -> WorkspaceSnapshotResult<Func> {
        let (node_weight, content) = self.func_get_node_weight_and_content(ctx, func_id).await?;

        Ok(Func::assemble(&node_weight, &content))
    }

    pub fn func_find_intrinsic(
        &mut self,
        intrinsic: IntrinsicFunc,
    ) -> WorkspaceSnapshotResult<FuncId> {
        let name = intrinsic.name();
        Ok(self
            .func_find_by_name(name)?
            .ok_or(WorkspaceSnapshotError::IntrinsicFuncNotFound(
                name.to_owned(),
            ))?)
    }

    pub async fn list_funcs(&mut self, ctx: &DalContext) -> WorkspaceSnapshotResult<Vec<Func>> {
        //      let start = Instant::now();
        let mut funcs = vec![];
        let (_, func_category_index) = self.working_copy()?.get_category(CategoryNodeKind::Func)?;

        let func_node_indexes = self.outgoing_targets_for_edge_weight_kind_by_index(
            func_category_index,
            EdgeWeightKindDiscriminants::Use,
        )?;

        let mut func_node_weights = vec![];
        let mut func_content_hash = vec![];
        for index in func_node_indexes {
            let node_weight = self.get_node_weight(index)?.get_func_node_weight()?;
            func_content_hash.push(node_weight.content_hash());
            func_node_weights.push(node_weight);
        }

        let func_contents: HashMap<ContentHash, FuncContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(func_content_hash.as_slice())
            .await?;

        for node_weight in func_node_weights {
            match func_contents.get(&node_weight.content_hash()) {
                Some(func_content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let FuncContent::V1(inner) = func_content;

                    funcs.push(Func::assemble(&node_weight, inner));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(funcs)
    }

    pub fn func_find_by_name(
        &mut self,
        name: impl AsRef<str>,
    ) -> WorkspaceSnapshotResult<Option<FuncId>> {
        let (_, func_category_index) = self.working_copy()?.get_category(CategoryNodeKind::Func)?;

        let func_id = self
            .working_copy()?
            .func_find_by_name(func_category_index, name)?;

        Ok(func_id.into())
    }

    pub async fn func_modify_by_id<L>(
        &mut self,
        ctx: &DalContext,
        id: FuncId,
        lambda: L,
    ) -> WorkspaceSnapshotResult<Func>
    where
        L: FnOnce(&mut Func) -> WorkspaceSnapshotResult<()>,
    {
        let (mut node_weight, content) = self.func_get_node_weight_and_content(ctx, id).await?;
        let mut func = Func::assemble(&node_weight, &content);

        lambda(&mut func)?;

        // If both either the name or backend_kind have changed, *and* parts of the FuncContent
        // have changed, this ends up updating the node for the function twice. This could be
        // optimized to do it only once.
        if func.name.as_str() != node_weight.name()
            || func.backend_kind != node_weight.backend_kind()
        {
            let original_node_index = self.working_copy()?.get_node_index_by_id(id.into())?;

            node_weight
                .set_name(func.name.as_str())
                .set_backend_kind(func.backend_kind);

            let new_node_index = self
                .working_copy()?
                .add_node(NodeWeight::Func(node_weight.clone()))?;

            self.working_copy()?
                .replace_references(original_node_index, new_node_index)?;
        }

        let updated = FuncContentV1::from(func.clone());
        if updated != content {
            let hash = ctx
                .content_store()
                .try_lock()?
                .add(&FuncContent::V1(updated.clone()))?;

            self.working_copy()?
                .update_content(ctx.change_set_pointer()?, id.into(), hash)?;
        }

        Ok(Func::assemble(&node_weight, &updated))
    }
}
