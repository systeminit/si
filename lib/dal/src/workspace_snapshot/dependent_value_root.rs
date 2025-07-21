use serde::{
    Deserialize,
    Serialize,
};
use si_id::ulid::Ulid;
use telemetry::prelude::*;
use thiserror::Error;

use super::{
    WorkspaceSnapshotError,
    node_weight::{
        NodeWeight,
        category_node_weight::CategoryNodeKind,
    },
};
use crate::{
    DalContext,
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
};

#[derive(Debug, Error)]
pub enum DependentValueRootError {
    #[error("Workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("transaction error: {0}")]
    Transactions(#[from] Box<crate::TransactionsError>),
}

impl From<crate::TransactionsError> for DependentValueRootError {
    fn from(value: crate::TransactionsError) -> Self {
        Box::new(value).into()
    }
}

pub type DependentValueRootResult<T> = Result<T, DependentValueRootError>;

#[derive(Copy, Clone, Hash, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependentValueRoot {
    Finished(Ulid),
    Unfinished(Ulid),
}

impl DependentValueRoot {
    pub fn is_finished(&self) -> bool {
        matches!(self, DependentValueRoot::Unfinished(_))
    }

    pub async fn add_dependent_value_root(
        ctx: &DalContext,
        root: Self,
    ) -> DependentValueRootResult<()> {
        let snap = ctx.workspace_snapshot().map_err(Box::new)?;

        if snap.dvu_root_check(root).await {
            return Ok(());
        }

        if let Some(dv_category_id) = snap
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)
            .await
            .map_err(Box::new)?
        {
            let id = snap.generate_ulid().await.map_err(Box::new)?;
            let lineage_id = snap.generate_ulid().await.map_err(Box::new)?;

            let node_weight = match root {
                DependentValueRoot::Finished(value_id) => {
                    NodeWeight::new_finished_dependent_value_root(id, lineage_id, value_id)
                }
                DependentValueRoot::Unfinished(value_id) => {
                    NodeWeight::new_dependent_value_root(id, lineage_id, value_id)
                }
            };

            snap.add_or_replace_node(node_weight)
                .await
                .map_err(Box::new)?;

            snap.add_edge(
                dv_category_id,
                EdgeWeight::new(EdgeWeightKind::new_use()),
                id,
            )
            .await
            .map_err(Box::new)?;
        }

        Ok(())
    }

    pub async fn roots_exist(ctx: &DalContext) -> DependentValueRootResult<bool> {
        let snap = ctx.workspace_snapshot().map_err(Box::new)?;

        Ok(
            match snap
                .get_category_node(None, CategoryNodeKind::DependentValueRoots)
                .await
                .map_err(Box::new)?
            {
                Some(dv_category_id) => !snap
                    .outgoing_targets_for_edge_weight_kind(
                        dv_category_id,
                        EdgeWeightKindDiscriminants::Use,
                    )
                    .await
                    .map_err(Box::new)?
                    .is_empty(),
                None => false,
            },
        )
    }

    /// Removes all the dependent value nodes from the category and returns the value_ids
    pub async fn take_dependent_values(ctx: &DalContext) -> DependentValueRootResult<Vec<Self>> {
        let snap = ctx.workspace_snapshot().map_err(Box::new)?;

        let dv_category_id = match snap
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)
            .await
            .map_err(Box::new)?
        {
            Some(cat_id) => cat_id,
            None => {
                return Ok(vec![]);
            }
        };

        let mut value_ids = vec![];
        let mut pending_removes = vec![];

        for dv_node_id in snap
            .outgoing_targets_for_edge_weight_kind(dv_category_id, EdgeWeightKindDiscriminants::Use)
            .await
            .map_err(Box::new)?
        {
            let root = match snap.get_node_weight(dv_node_id).await.map_err(Box::new)? {
                NodeWeight::DependentValueRoot(unfinished) => Some((
                    DependentValueRoot::Unfinished(unfinished.value_id()),
                    unfinished.id(),
                )),
                NodeWeight::FinishedDependentValueRoot(finished) => Some((
                    DependentValueRoot::Finished(finished.value_id()),
                    finished.id(),
                )),
                _ => None,
            };

            if let Some((root, node_weight_id)) = root {
                value_ids.push(root);
                pending_removes.push(node_weight_id);
            }
        }

        for to_remove_id in pending_removes {
            snap.remove_node_by_id(to_remove_id)
                .await
                .map_err(Box::new)?;
        }

        Ok(value_ids)
    }

    /// List all the `value_ids` from the dependent value nodes in the category.
    pub async fn get_dependent_value_roots(
        ctx: &DalContext,
    ) -> DependentValueRootResult<Vec<Self>> {
        let snap = ctx.workspace_snapshot().map_err(Box::new)?;

        let dv_category_id = match snap
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)
            .await
            .map_err(Box::new)?
        {
            Some(cat_id) => cat_id,
            None => {
                return Ok(vec![]);
            }
        };

        let mut roots = vec![];
        for dv_node_idx in snap
            .outgoing_targets_for_edge_weight_kind(dv_category_id, EdgeWeightKindDiscriminants::Use)
            .await
            .map_err(Box::new)?
        {
            match snap.get_node_weight(dv_node_idx).await.map_err(Box::new)? {
                NodeWeight::DependentValueRoot(unfinished) => {
                    roots.push(DependentValueRoot::Unfinished(unfinished.value_id()));
                }
                NodeWeight::FinishedDependentValueRoot(finished) => {
                    roots.push(DependentValueRoot::Finished(finished.value_id()));
                }
                _ => {}
            }
        }

        Ok(roots)
    }
}

impl From<DependentValueRoot> for Ulid {
    fn from(value: DependentValueRoot) -> Self {
        match value {
            DependentValueRoot::Finished(id) | DependentValueRoot::Unfinished(id) => id,
        }
    }
}
