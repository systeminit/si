use dal::DalContext;
use si_frontend_mv_types::object::patch::{
    IndexUpdate,
    PatchBatch,
};
use telemetry::prelude::*;
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DataCacheError {
    #[error("Nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("Transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
}

type Result<T> = std::result::Result<T, DataCacheError>;

pub(crate) struct DataCache;

impl DataCache {
    #[instrument(name = "data_cache.publish_patch_batch", level = "info", skip_all)]
    pub async fn publish_patch_batch(ctx: &DalContext, patch_batch: PatchBatch) -> Result<()> {
        ctx.txns()
            .await?
            .nats()
            .publish_immediately(patch_batch.publish_subject(), &patch_batch)
            .await?;

        Ok(())
    }
    #[instrument(name = "data_cache.publish_index_update", level = "debug", skip_all)]
    pub async fn publish_index_update(ctx: &DalContext, index_update: IndexUpdate) -> Result<()> {
        ctx.txns()
            .await?
            .nats()
            .publish_immediately(index_update.publish_subject(), &index_update)
            .await?;

        Ok(())
    }
}
