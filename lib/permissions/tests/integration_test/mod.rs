use std::env;

use indoc::indoc;
use permissions::{
    ObjectType,
    Permission,
    PermissionBuilder,
    Relation,
    RelationBuilder,
};
use rand::{
    Rng,
    distributions::Alphanumeric,
    thread_rng,
};
use si_data_spicedb::{
    Client,
    SpiceDbClient,
    SpiceDbConfig,
};

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

async fn write_schema(mut client: SpiceDbClient) {
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
}

#[tokio::test]
async fn add_remove_approver_from_workspace() {
    let config = spicedb_config();

    let mut client = Client::new(&config)
        .await
        .expect("failed to connect to spicedb");

    write_schema(client.clone()).await;

    let user_id = "scott".to_string();
    let workspace_id = "123".to_string();

    let relation = RelationBuilder::new()
        .object(ObjectType::Workspace, workspace_id.clone())
        .relation(Relation::Approver)
        .subject(ObjectType::User, user_id.clone());

    let zed_token = relation
        .create(&mut client)
        .await
        .expect("could not create relationship")
        .expect("could not unwrap zed token");

    let can_approve = PermissionBuilder::new()
        .object(ObjectType::Workspace, workspace_id.clone())
        .permission(Permission::Approve)
        .subject(ObjectType::User, user_id.clone())
        .zed_token(zed_token);

    assert!(
        can_approve
            .has_permission(&mut client)
            .await
            .expect("could not check permission")
    );

    let zed_token = relation
        .delete(&mut client)
        .await
        .expect("could not delete permission")
        .expect("could not unwrap zed token");

    assert!(
        !can_approve
            .zed_token(zed_token)
            .has_permission(&mut client)
            .await
            .expect("could not check permission")
    );
}
