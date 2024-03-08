#![recursion_limit = "256"]

const TEST_PG_DBNAME: &str = "si_test_sdf_server";
const TEST_CONTENT_STORE_PG_DBNAME: &str = "si_test_content_store";
const SI_TEST_LAYER_CACHE_PG_DBNAME: &str = "si_test_key_value_pairs";

mod service_tests;
