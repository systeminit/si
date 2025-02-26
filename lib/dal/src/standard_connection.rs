use anyhow::Result;
use thiserror::Error;

use crate::{
    EdgeWeightKind, EdgeWeightKindDiscriminants, TransactionsError, WorkspaceSnapshotError,
};

#[derive(Debug, Error)]
pub enum HelperError {
    #[error("invalid edge weight, got {0:?} expected discriminant {1:?}")]
    InvalidEdgeWeight(EdgeWeightKind, EdgeWeightKindDiscriminants),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type HelperResult<T> = Result<T>;

#[macro_export]
macro_rules! implement_add_edge_to {
    (
        source_id: $source_id:ty,
        destination_id: $destination_id:ty,
        add_fn: $add_fn:ident,
        discriminant: $discriminant:expr,
        result: $result:ty,
    ) => {
        paste::paste! {
            /// Inserts edge from source to destination with specified weight to the graph
            pub async fn $add_fn(ctx: &$crate::DalContext, source_id: $source_id, destination_id: $destination_id, weight: $crate::EdgeWeightKind) -> ::anyhow::Result<()> {
                if $crate::EdgeWeightKindDiscriminants::from(&weight) != $discriminant {
                    return Err($crate::HelperError::InvalidEdgeWeight(weight, $discriminant))?;
                }

                ctx.workspace_snapshot()?
                    .add_edge(
                        source_id,
                        $crate::EdgeWeight::new(weight),
                        destination_id,
                    )
                    .await?;
                Ok(())
            }

            /// Inserts ordered edge from source to destination with specified weight to the graph
            #[allow(dead_code)]
            pub async fn [<$add_fn _ordered>](ctx: &DalContext, source_id: $source_id, destination_id: $destination_id, weight: $crate::EdgeWeightKind) -> ::anyhow::Result<()> {
                if $crate::EdgeWeightKindDiscriminants::from(&weight) != $discriminant {
                    return Err($crate::HelperError::InvalidEdgeWeight(weight, $discriminant))?;
                }

                ctx.workspace_snapshot()?
                    .add_ordered_edge(
                        source_id,
                        $crate::EdgeWeight::new(weight),
                        destination_id
                    )
                    .await?;
                Ok(())
            }
        }
    }
}
