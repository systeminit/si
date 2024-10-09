use std::env;

use indoc::indoc;
use si_data_spicedb::{Client, SpiceDbConfig};

const ENV_VAR_SPICEDB_URL: &str = "SI_TEST_SPICEDB_URL";

fn spicedb_config() -> SpiceDbConfig {
    let mut config = SpiceDbConfig::default();
    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = env::var(ENV_VAR_SPICEDB_URL) {
        config.endpoint = value.parse().expect("failed to parse spicedb url");
    }
    config
}

#[tokio::test]
async fn write_and_read_schema() {
    let config = spicedb_config();

    let mut client = Client::new(&config)
        .await
        .expect("failed to connect to spicedb");

    let schema = indoc! {"
        // Plan comment
        definition plan {}

        definition user {}

        definition workspace {
            relation approver: user
            permission approve = approver
        }
    "};

    client
        .write_schema(schema)
        .await
        .expect("failed to write schema");

    let response = client.read_schema().await.expect("failed to read schema");

    assert!(response
        .schema_text()
        .lines()
        .any(|line| line == "// Plan comment"));
    assert!(response
        .schema_text()
        .lines()
        .any(|line| line == "definition plan {}"));
}
