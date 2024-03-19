pub use test_exclusive_schema_dummy_secret::migrate_test_exclusive_schema_dummy_secret;
pub use test_exclusive_schema_fallout::migrate_test_exclusive_schema_fallout;
pub use test_exclusive_schema_katy_perry::migrate_test_exclusive_schema_katy_perry;
pub use test_exclusive_schema_pirate::migrate_test_exclusive_schema_pirate;
pub use test_exclusive_schema_starfield::migrate_test_exclusive_schema_starfield;
pub use test_exclusive_schema_swifty::migrate_test_exclusive_schema_swifty;

const PKG_VERSION: &str = "2019-06-03";
const PKG_CREATED_BY: &str = "System Initiative";

mod schema_helpers;
mod test_exclusive_schema_dummy_secret;
mod test_exclusive_schema_fallout;
mod test_exclusive_schema_katy_perry;
mod test_exclusive_schema_pirate;
mod test_exclusive_schema_starfield;
mod test_exclusive_schema_swifty;
