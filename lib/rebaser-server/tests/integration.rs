//! All tests should be ran with the following environment variable:
//!
//! ```shell
//! SI_TEST_BUILTIN_SCHEMAS=none
//! ```

const TEST_PG_DBNAME: &str = "si_test_rebaser";

mod integration_test;
