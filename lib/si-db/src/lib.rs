//! DAL types that are stored in SQL.

pub mod actor_view;
pub mod change_set;
pub mod context;
pub mod history_event;
pub mod key_pair;
pub mod migrate;
// TODO remove pub once we move users out of dal
pub mod standard_accessors;
pub mod tenancy;
pub mod user;
pub mod visibility;
pub mod workspace;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}
