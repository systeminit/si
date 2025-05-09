use std::{
    collections::{
        HashMap,
        HashSet,
    },
    result,
    sync::Arc,
};

use dal::{
    AccessBuilder,
    ChangeSet,
    DalContext,
    DalContextBuilder,
};
use frigg::FriggStore;
use futures::{
    StreamExt,
    TryStreamExt,
};
use json_patch::jsonptr::index;
use si_events::authentication_method::AuthenticationMethodV1;
use si_frontend_types::index::MvIndex;
use si_id::WorkspacePk;
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::ServerMetadata;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum GarbageCollectWorkspaceTaskError {
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] dal::TransactionsError),
    #[error("frigg error: {0}")]
    Frig(#[from] frigg::Error),
}

type Error = GarbageCollectWorkspaceTaskError;

type Result<T> = result::Result<T, GarbageCollectWorkspaceTaskError>;

pub(crate) struct GarbageCollectWorkspaceTask {
    _metadata: Arc<ServerMetadata>,
    workspace_id: WorkspacePk,
    ctx_builder: DalContextBuilder,
    frigg: FriggStore,
    token: CancellationToken,
}

impl GarbageCollectWorkspaceTask {
    const NAME: &'static str = "edda_server::garbage_collect_workspace_task";

    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        workspace_id: WorkspacePk,
        ctx_builder: DalContextBuilder,
        frigg: FriggStore,
        token: CancellationToken,
    ) -> Self {
        Self {
            _metadata: metadata,
            workspace_id,
            ctx_builder,
            frigg,
            token,
        }
    }

    pub(crate) async fn try_run(self) -> Result<()> {
        let ctx = self.build_ctx_on_head().await?;

        // Hash set of active change set ids
        let change_set_ids: HashSet<_> = ChangeSet::list_active(&ctx)
            .await?
            .iter()
            .map(|change_set| change_set.id)
            .collect();
        // Hash map from change set ids to index keys
        let frigg_index_keys: HashMap<_, _> = self
            .frigg
            .index_keys_for_workspace(self.workspace_id)
            .await?
            // Convert in tuples of `(change_set_id, index_key)` which we'll collect into a HashMap
            .map_ok(|index_key| (index_key.change_set_id(), index_key))
            .try_collect()
            .await?;

        // Split into 2 maps containing active index keys and inactive (i.e. old) index keys to be
        // deleted
        let (active_index_keys, old_index_keys): (HashMap<_, _>, HashMap<_, _>) = frigg_index_keys
            .into_iter()
            .partition(|(change_set_id, _)| change_set_ids.contains(change_set_id));

        dbg!(&active_index_keys);
        dbg!(&old_index_keys);

        // Delete all index objects and pointer keys for change sets that are no longer active
        for (_, old_index_key) in old_index_keys {
            self.frigg
                .delete_index(old_index_key.workspace_id(), old_index_key.change_set_id())
                .await?;
        }

        let frigg_object_keys: HashMap<_, _> = self
            .frigg
            .object_keys_for_workspace(self.workspace_id)
            .await?
            // Convert in tuples of `(index_reference, object_key)` which we'll collect into a
            // HashMap
            .map_ok(|object_key| (object_key.to_index_reference(), object_key))
            .try_collect()
            .await?;

        for (_, active_index_key) in active_index_keys {
            let mv_index = self
                .frigg
                .get_mv_index(
                    active_index_key.workspace_id(),
                    active_index_key.change_set_id(),
                )
                .await?;
        }

        debug!(
            task = Self::NAME,
            si.workspace.id = %self.workspace_id,
            "shutdown complete",
        );

        Ok(())
    }

    async fn build_ctx_on_head(&self) -> Result<DalContext> {
        let access_builder = AccessBuilder::new(
            self.workspace_id.into(),
            dal::HistoryActor::SystemInit,
            None,
            AuthenticationMethodV1::System,
        );

        self.ctx_builder
            .build_head(access_builder)
            .await
            .map_err(Into::into)
    }
}
