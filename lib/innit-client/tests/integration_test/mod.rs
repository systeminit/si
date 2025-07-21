mod certs;
#[cfg(test)]
mod integration_tests {

    use std::{
        env,
        net::SocketAddr,
        sync::Once,
    };

    use base64::{
        Engine as _,
        engine::general_purpose::STANDARD,
    };
    use hyper::server::conn::AddrIncoming;
    use innit_client::{
        InnitClient,
        InnitClientError,
        auth::AuthConfig,
        config::StandardConfig,
    };
    use innit_server::Server;
    use si_tls::{
        CertificateSource,
        KeySource,
    };
    use tokio_util::sync::CancellationToken;
    use url::Url;

    use super::certs::*;

    struct TestCerts {
        ca_cert: String,
        client_cert: String,
        client_key: String,
    }

    const ENV_VAR_LOCALSTACK_URL: &str = "SI_TEST_LOCALSTACK_URL";
    const DEFAULT_ENDPOINT: &str = "http://localhost:4566";

    // a crypto provider (only one) is required to generate certs, which happens in these tests
    static INIT: Once = Once::new();
    fn initialize_crypto() {
        INIT.call_once(|| {
            rustls::crypto::ring::default_provider()
                .install_default()
                .expect("Failed to install rustls crypto provider");
        });
    }

    fn create_certs() -> TestCerts {
        initialize_crypto();

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

        let mut endpoint = DEFAULT_ENDPOINT.to_string();
        #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
        if let Ok(value) = env::var(ENV_VAR_LOCALSTACK_URL) {
            endpoint = value;
        }

        let config = innit_server::Config::builder()
            .socket_addr(addr)
            .client_ca_certs(Some(vec![CertificateSource::Base64(ca_cert)]))
            .test_endpoint(Some(endpoint))
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

        // should 200
        let result = client.check_health().await;
        assert!(
            result.is_ok(),
            "Health is a public endpoint, this should succeed"
        );

        // should 401
        let result = client.get_parameter("/fake".to_string()).await;
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

    #[tokio::test]
    async fn create_param() {
        let certs = create_certs();
        let addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let (server, token) = setup_test_server(addr, certs.ca_cert).await;
        let bound_port = server.bound_port();
        let server_handle =
            tokio::spawn(async move { server.run().await.expect("should run server") });

        // Create client with proper auth certs
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

        let param_name = "/si/test/test/param1";
        let initial_value = "initial_value";
        let updated_value = "updated_value";

        // create a parameter
        client
            .create_parameter(param_name.to_string(), initial_value.to_string())
            .await
            .expect("should create parameter");

        // get it, should be cached
        let get_response = client
            .get_parameter(param_name.to_string())
            .await
            .expect("should get parameter");

        assert_eq!(
            get_response.parameter.value,
            Some(initial_value.to_string())
        );
        assert!(get_response.is_cached);

        // update it
        client
            .create_parameter(param_name.to_string(), updated_value.to_string())
            .await
            .expect("should update parameter");

        // get it, should be cached
        let get_response = client
            .get_parameter(param_name.to_string())
            .await
            .expect("should get updated parameter");

        assert_eq!(
            get_response.parameter.value,
            Some(updated_value.to_string())
        );
        assert!(get_response.is_cached);

        // clear the cache
        let refresh_response = client
            .clear_parameter_cache()
            .await
            .expect("should refresh cache");

        assert!(refresh_response.success, "Cache refresh should succeed");

        // get again, ensure not cached
        let get_response = client
            .get_parameter(param_name.to_string())
            .await
            .expect("should get parameter");

        assert_eq!(
            get_response.parameter.value,
            Some(updated_value.to_string())
        );
        assert!(!get_response.is_cached);

        token.cancel();
        server_handle.abort();
    }

    #[tokio::test]
    async fn create_and_get_by_path() {
        let certs = create_certs();
        let addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let (server, token) = setup_test_server(addr, certs.ca_cert).await;
        let bound_port = server.bound_port();
        let server_handle =
            tokio::spawn(async move { server.run().await.expect("should run server") });

        // Create client with proper auth certs
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

        let path_prefix = "/si/test/test";

        // create two parameters in the same path
        client
            .create_parameter(
                format!("{path_prefix}/param1"),
                "path_test_value".to_string(),
            )
            .await
            .expect("should create parameter");

        client
            .create_parameter(
                format!("{path_prefix}/param2"),
                "path_test_value".to_string(),
            )
            .await
            .expect("should create second parameter");

        // clear the cache
        let refresh_response = client
            .clear_parameter_cache()
            .await
            .expect("should refresh cache");

        assert!(refresh_response.success, "Cache refresh should succeed");

        // get parameters by path, no cache
        let list_response = client
            .get_parameters_by_path(path_prefix.to_string())
            .await
            .expect("should list parameters");

        assert_eq!(
            list_response.parameters.len(),
            2,
            "Should find two parameters"
        );
        assert!(!list_response.is_cached);

        // Make a second request which should be served from cache
        let cached_list_response = client
            .get_parameters_by_path(path_prefix.to_string())
            .await
            .expect("should list parameters from cache");

        assert_eq!(
            cached_list_response.parameters.len(),
            2,
            "Should find two parameters from cache"
        );
        assert!(cached_list_response.is_cached);

        token.cancel();
        server_handle.abort();
    }
}
