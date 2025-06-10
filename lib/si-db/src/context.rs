use async_trait::async_trait;
use si_id::ChangeSetId;
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
}
