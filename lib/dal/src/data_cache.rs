use si_frontend_types::object::patch::PatchBatch;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{DalContext, TransactionsError};

pub type DataCacheResult<T> = Result<T, DataCacheError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DataCacheError {
    #[error("Nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub struct DataCache;

impl DataCache {
    #[instrument(name = "data_cache.publish_patch_batch", level = "info", skip_all)]
    pub async fn publish_patch_batch(
        ctx: &DalContext,
        patch_batch: PatchBatch,
    ) -> DataCacheResult<()> {
        ctx.txns()
            .await?
            .nats()
            .publish(patch_batch.publish_subject(), &patch_batch)
            .await?;

        Ok(())
    }
}
