//! DAL types that are stored in SQL.

use si_id::UserPk;

mod actor_view;
pub mod change_set;
mod context;
mod func_run;
mod func_run_log;
mod history_event;
pub mod key_pair;
mod management_func_execution;
pub mod migrate;
mod policy_report;
// TODO remove pub once we move users out of dal
pub mod standard_accessors;
mod tenancy;
mod transactions;
mod user;
mod visibility;
pub mod workspace;

pub use actor_view::ActorView;
pub use context::SiDbContext;
pub use func_run::FuncRunDb;
pub use func_run_log::FuncRunLogDb;
pub use history_event::{
    HistoryActor,
    HistoryEvent,
    HistoryEventMetadata,
};
pub use management_func_execution::{
    ManagementFuncExecutionError,
    ManagementFuncJobState,
    ManagementState,
};
pub use policy_report::{
    DEFAULT_PAGE_NUMBER,
    DEFAULT_PAGE_SIZE,
    MAX_REPORTS_PER_GROUP,
    PolicyReport,
    PolicyReportError,
    PolicyReportResult,
};
pub use tenancy::Tenancy;
pub use transactions::{
    SiDbTransactions,
    SiDbTransactionsError,
};
pub use user::User;
pub use visibility::Visibility;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

#[remain::sorted]
#[derive(thiserror::Error, Debug)]
pub enum SiDbError {
    #[error("action id not found: {0}")]
    ActionIdNotFound(si_events::ActionId),
    #[error("missing func run: {0}")]
    MissingFuncRun(si_events::FuncRunId),
    #[error("nats error")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no workspace")]
    NoWorkspace,
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("postcard error: {0}")]
    Postcard(String),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db transactions error: {0}")]
    SiDbTransactions(#[from] transactions::SiDbTransactionsError),
    #[error("user not found: {0}")]
    UserNotFound(UserPk),
}

pub type SiDbResult<T> = Result<T, SiDbError>;
