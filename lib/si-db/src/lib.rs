//! DAL types that are stored in SQL.

use si_id::UserPk;

mod actor_view;
pub mod change_set;
mod context;
mod history_event;
pub mod key_pair;
pub mod migrate;
// TODO remove pub once we move users out of dal
pub mod standard_accessors;
mod tenancy;
mod transactions;
mod user;
mod visibility;
pub mod workspace;

pub use actor_view::ActorView;
pub use context::SiDbContext;
pub use history_event::{
    HistoryActor,
    HistoryEvent,
    HistoryEventMetadata,
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
pub enum Error {
    #[error("nats error")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no workspace")]
    NoWorkspace,
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db transactions error: {0}")]
    SiDbTransactions(#[from] transactions::SiDbTransactionsError),
    #[error("user not found: {0}")]
    UserNotFound(UserPk),
}

pub type Result<T> = std::result::Result<T, Error>;
