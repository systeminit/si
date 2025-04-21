use std::env;

use indoc::indoc;
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use si_data_spicedb::{Client, Permission, Relationship, SpiceDBObject, SpiceDbConfig};

const ENV_VAR_SPICEDB_URL: &str = "SI_TEST_SPICEDB_URL";

fn spicedb_config() -> SpiceDbConfig {
    let mut config = SpiceDbConfig::default();
    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = env::var(ENV_VAR_SPICEDB_URL) {
        config.endpoint = value.parse().expect("failed to parse spicedb url");
    }

    let mut rng = thread_rng();
    let random_string: String = (0..12).map(|_| rng.sample(Alphanumeric) as char).collect();
    config.preshared_key = random_string.into();
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

    assert!(
        response
            .schema_text()
            .lines()
            .any(|line| line == "// Plan comment")
    );
    assert!(
        response
            .schema_text()
            .lines()
            .any(|line| line == "definition plan {}")
    );
}

#[tokio::test]
async fn write_and_read_relationship() {
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

    let workspace_object = SpiceDBObject::new("workspace", "456".to_string());
    let user_object = SpiceDBObject::new("user", "scott".to_string());
    let mut scott_aprover_workspace =
        Relationship::new(workspace_object, "approver", user_object, None);

    let zed_token = client
        .create_relationships(vec![scott_aprover_workspace.clone()])
        .await
        .expect("failed to create a relation");

    scott_aprover_workspace.set_zed_token(zed_token);

    let resp = client
        .read_relationship(scott_aprover_workspace.clone())
        .await
        .expect("failed to read relation");

    assert!(resp.len() == 1);

    let zed_token = client
        .delete_relationships(vec![scott_aprover_workspace.clone()])
        .await
        .expect("failed to delete relation");

    scott_aprover_workspace.set_zed_token(zed_token);

    let resp = client
        .read_relationship(scott_aprover_workspace.clone())
        .await
        .expect("failed to read relation");

    assert!(resp.is_empty());
}

#[tokio::test]
async fn check_permissions() {
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

    let workspace_object = SpiceDBObject::new("workspace", "789".to_string());
    let user_object = SpiceDBObject::new("user", "scott".to_string());
    let user_object2 = SpiceDBObject::new("user", "fletcher".to_string());
    let scott_aprover_workspace = Relationship::new(
        workspace_object.clone(),
        "approver",
        user_object.clone(),
        None,
    );

    let zed_token = client
        .create_relationships(vec![scott_aprover_workspace.clone()])
        .await
        .expect("failed to create a relation");

    let perms = Permission::new(
        workspace_object.clone(),
        "approve",
        user_object.clone(),
        zed_token.clone(),
    );
    let bad_perms = Permission::new(workspace_object, "approve", user_object2, zed_token);

    assert!(
        client
            .check_permissions(perms)
            .await
            .expect("failed to check permissions")
    );

    assert!(
        !client
            .check_permissions(bad_perms)
            .await
            .expect("failed to check permissions")
    );
    let users = client
        .lookup_subjects(
            "workspace".to_string(),
            "789".to_string(),
            "approve".to_string(),
            "user".to_string(),
        )
        .await
        .expect("could not list subjects");
    assert!(users.contains(&"scott".to_string()));
    assert!(!users.contains(&"fletcher".to_string()));
}
