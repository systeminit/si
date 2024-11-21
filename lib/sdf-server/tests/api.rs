#![recursion_limit = "256"]

const TEST_PG_DBNAME: &str = "si_test_sdf_server";
const SI_TEST_LAYER_CACHE_PG_DBNAME: &str = "si_test_layer_db";
const SI_TEST_AUDIT_PG_DBNAME: &str = "si_test_audit";

mod service_tests;
