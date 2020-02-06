/// The Migrateable trait is something that can be migrated and managed as a versioned
/// object in the database.
pub trait Migrateable: std::fmt::Debug {
    fn get_version(&self) -> i32;
}
