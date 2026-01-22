use async_trait::async_trait;
use si_id::ChangeSetId;
use si_layer_cache::db::{
    func_run::FuncRunLayerDb,
    func_run_log::FuncRunLogLayerDb,
};
use tokio::sync::MappedMutexGuard;

use crate::{
    history_event::HistoryActor,
    tenancy::Tenancy,
    transactions::{
        SiDbTransactions,
        SiDbTransactionsError,
    },
    visibility::Visibility,
};

#[async_trait]
pub trait SiDbContext {
    type Transactions: SiDbTransactions;
    fn history_actor(&self) -> &HistoryActor;
    async fn txns(&self)
    -> Result<MappedMutexGuard<'_, Self::Transactions>, SiDbTransactionsError>;
    fn tenancy(&self) -> &Tenancy;
    fn visibility(&self) -> &Visibility;
    fn change_set_id(&self) -> ChangeSetId;
    // TODO get rid of these after we don't need layer db fallbacks
    fn func_run_layer_db(&self) -> &FuncRunLayerDb;
    fn func_run_log_layer_db(&self) -> &FuncRunLogLayerDb;
}
