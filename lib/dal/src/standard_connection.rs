use crate::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants, TransactionsError,
    WorkspaceSnapshotError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HelperError {
    #[error("invalid edge weight, got {0:?} expected discriminant {1:?}")]
    InvalidEdgeWeight(EdgeWeightKind, EdgeWeightKindDiscriminants),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type HelperResult<T> = Result<T, HelperError>;

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
            pub async fn $add_fn(ctx: &$crate::DalContext, source_id: $source_id, destination_id: $destination_id, weight: $crate::EdgeWeightKind) -> $result<()> {
                if $crate::EdgeWeightKindDiscriminants::from(&weight) != $discriminant {
                    return Err($crate::HelperError::InvalidEdgeWeight(weight, $discriminant))?;
                }

                ctx.workspace_snapshot()?
                    .add_edge(
                        source_id,
                        $crate::EdgeWeight::new(ctx.change_set()?, weight)?,
                        destination_id,
                    )
                    .await?;
                Ok(())
            }

            /// Inserts ordered edge from source to destination with specified weight to the graph
            pub async fn [<$add_fn _ordered>](ctx: &DalContext, source_id: $source_id, destination_id: $destination_id, weight: $crate::EdgeWeightKind) -> $result<()> {
                if $crate::EdgeWeightKindDiscriminants::from(&weight) != $discriminant {
                    return Err($crate::HelperError::InvalidEdgeWeight(weight, $discriminant))?;
                }

                ctx.workspace_snapshot()?
                    .add_ordered_edge(
                        ctx.change_set()?,
                        source_id,
                        $crate::EdgeWeight::new(ctx.change_set()?, weight)?,
                        destination_id
                    )
                    .await?;
                Ok(())
            }
        }
    }
}
