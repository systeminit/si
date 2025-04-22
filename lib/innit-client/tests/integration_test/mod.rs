mod certs;
#[cfg(test)]
mod integration_tests {

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use innit_client::InnitClientError;
    use std::net::SocketAddr;

    use hyper::server::conn::AddrIncoming;
    use innit_client::config::StandardConfig;
    use innit_client::{InnitClient, auth::AuthConfig};
    use innit_server::Server;
    use si_tls::{CertificateSource, KeySource};
    use tokio_util::sync::CancellationToken;
    use url::Url;

    use super::certs::*;

    struct TestCerts {
        ca_cert: String,
        client_cert: String,
        client_key: String,
    }

    fn create_certs() -> TestCerts {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
        let (ca, ca_key) = new_ca();
        let (client_cert, client_key) = new_end_entity(&ca, &ca_key);

        TestCerts {
            ca_cert: STANDARD.encode(ca.pem()),
            client_cert: STANDARD.encode(client_cert.pem()),
            client_key: STANDARD.encode(client_key.serialize_pem()),
        }
    }

    async fn setup_test_server(
        addr: SocketAddr,
        ca_cert: String,
    ) -> (Server<AddrIncoming>, CancellationToken) {
        let token = CancellationToken::new();
        let config = innit_server::Config::builder()
            .socket_addr(addr)
            .client_ca_cert(Some(CertificateSource::Base64(ca_cert)))
            .build()
            .expect("should build config");
        let server = Server::http(config, token.clone())
            .await
            .expect("should create server");
        (server, token)
    }

    #[tokio::test]
    async fn test_client_server_mtls_interaction() {
        let certs = create_certs();
        let addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let (server, token) = setup_test_server(addr, certs.ca_cert).await;
        let bound_port = server.bound_port();
        let server_handle =
            tokio::spawn(async move { server.run().await.expect("should make run server") });

        // client without auth certs
        let client_config = innit_client::config::Config::builder()
            .base_url(
                Url::parse(&format!("http://{}:{}", &addr.ip(), bound_port))
                    .expect("should parse addr"),
            )
            .auth_config(AuthConfig {
                client_cert: None,
                client_key: None,
            })
            .build()
            .expect("should make new config");

        let client = InnitClient::new(client_config)
            .await
            .expect("should make new client");

        // should 401
        let result = client.check_health().await;
        assert!(result.is_err(), "Expected an error but got success");
        match result.unwrap_err() {
            InnitClientError::Request(err) => {
                dbg!(&err);
                assert_eq!(
                    err.status().expect("should unwrap the status").as_u16(),
                    401,
                    "Expected 401 status code"
                );
            }
            err => panic!("Unexpected error type: {err}"),
        }

        // new client with certs signed by the server ca
        let client_config = innit_client::config::Config::builder()
            .base_url(
                Url::parse(&format!("http://{}:{}", &addr.ip(), bound_port))
                    .expect("should parse addr"),
            )
            .auth_config(AuthConfig {
                client_cert: Some(CertificateSource::Base64(certs.client_cert)),
                client_key: Some(KeySource::Base64(certs.client_key)),
            })
            .build()
            .expect("should make new config");

        let client = InnitClient::new(client_config)
            .await
            .expect("should make new client");

        // should auth and be ok
        assert!(client.check_health().await.expect("should be ok").ok);

        token.cancel();
        server_handle.abort();
    }
}
