use indoc::indoc;
use si_data_spicedb::{Client, SpiceDbConfig};

#[tokio::test]
async fn write_and_read_schema() {
    let config = SpiceDbConfig::default();

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
