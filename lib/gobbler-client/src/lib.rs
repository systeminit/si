//! This crate provides the gobbler [`Client`], which is used for communicating with a running
//! gobbler [`Server`](gobbler_server::Server).

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

mod client;

pub use client::Client;

use si_rabbitmq::RabbitError;
use telemetry::prelude::error;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("gobbler stream for change set not found")]
    GobblerStreamForChangeSetNotFound,
    #[error("si rabbitmq error: {0}")]
    Rabbit(#[from] RabbitError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[allow(missing_docs)]
pub type ClientResult<T> = Result<T, ClientError>;

#[cfg(test)]
mod tests {
    use super::*;
    use gobbler_server::{ConfigBuilder, Server};
    use tokio::test;
    use ulid::Ulid;

    async fn test_setup() -> Client {
        // FIXME(nick): make this not brittle... make strong!
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

    #[test]
    async fn connect() {
        let client = test_setup().await;
        client.close().await;
    }

    #[test]
    async fn send_management() {
        let mut client = test_setup().await;

        let change_set_id = Ulid::new();
        let _new_stream_to_produce_to = client
            .send_management_open(change_set_id)
            .await
            .expect("could not create new gobbler loop for change set")
            .expect("no message returned");

        client
            .send_management_close(change_set_id)
            .await
            .expect("could not close the gobbler loop for change set");

        client.close().await;
    }

    #[test]
    async fn send_management_and_round_trip() {
        let mut client = test_setup().await;

        let change_set_id = Ulid::new();
        let _new_stream_to_produce_to = client
            .send_management_open(change_set_id)
            .await
            .expect("could not create new gobbler loop for change set")
            .expect("no message returned");

        let contents = "MUSTANG GTD";
        let message = client
            .send_with_reply(contents, change_set_id)
            .await
            .expect("could not send message")
            .expect("no message returned");
        assert_eq!(contents, &message);

        client
            .send_management_close(change_set_id)
            .await
            .expect("could not close the gobbler loop for change set");

        client.close().await;
    }
}
