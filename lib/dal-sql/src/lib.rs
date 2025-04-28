//! DAL types that are stored in SQL.

pub mod change_set;
pub mod key_pair;
pub mod migrate;
pub mod user;
pub mod workspace;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}
