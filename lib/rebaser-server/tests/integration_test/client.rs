use tokio::test;
use ulid::Ulid;

use rebaser_client::Client;
use rebaser_server::{ConfigBuilder, Server};

#[test]
async fn connect() {
    let client = test_setup().await;
    client.close().await;
}

#[test]
async fn management() {
    let mut client = test_setup().await;

    let change_set_id = Ulid::new();
    let _new_stream_to_produce_to = client
        .send_management_open_change_set(change_set_id)
        .await
        .expect("could not create new rebaser loop for change set");

    client
        .send_management_close_change_set(change_set_id)
        .await
        .expect("could not close the rebaser loop for change set");

    client.close().await;
}

async fn test_setup() -> Client {
    let config = ConfigBuilder::default()
        .cyclone_encryption_key_path(
            "../../lib/cyclone-server/src/dev.encryption.key"
                .try_into()
                .expect("could not convert"),
        )
        .build()
        .expect("could not build config");
    let server = Server::from_config(config)
        .await
        .expect("could not build server");
    tokio::spawn(server.run());

    Client::new().await.expect("could not build client")
}
