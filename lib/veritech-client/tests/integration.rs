use std::env;

use base64::{engine::general_purpose, Engine};
use cyclone_core::{
    ComponentKind, ComponentView, FunctionResult, ResolverFunctionComponent,
    ResolverFunctionRequest, ResolverFunctionResponseType, SchemaVariantDefinitionRequest,
    ValidationRequest,
};
use si_data_nats::{NatsClient, NatsConfig};
use test_log::test;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;
use uuid::Uuid;
use veritech_client::Client;
use veritech_server::{
    Config, CycloneSpec, Instance, LocalUdsInstance, Server, ServerError, StandardConfig,
};

fn nats_config(subject_prefix: String) -> NatsConfig {
    let mut config = NatsConfig::default();
    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = env::var("SI_TEST_NATS_URL") {
        config.url = value;
    }
    config.subject_prefix = Some(subject_prefix);
    config
}

async fn nats(subject_prefix: String) -> NatsClient {
    NatsClient::new(&nats_config(subject_prefix))
        .await
        .expect("failed to connect to NATS")
}

fn nats_prefix() -> String {
    Uuid::new_v4().as_simple().to_string()
}

async fn veritech_server_for_uds_cyclone(subject_prefix: String) -> Server {
    let mut config_file = veritech_server::ConfigFile::default_local_uds();
    veritech_server::detect_and_configure_development(&mut config_file)
        .expect("failed to determine test configuration");

    let cyclone_spec = CycloneSpec::LocalUds(
        LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to setup cyclone_cmd_path")
            .cyclone_decryption_key_path(config_file.cyclone.cyclone_decryption_key_path())
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to setup lang_js_cmd_path")
            .all_endpoints()
            .build()
            .expect("failed to build cyclone spec"),
    );
    let config = Config::builder()
        .nats(nats_config(subject_prefix.clone()))
        .cyclone_spec(cyclone_spec)
        .build()
        .expect("failed to build spec");
    Server::for_cyclone_uds(config)
        .await
        .expect("failed to create server")
}

async fn client(subject_prefix: String) -> Client {
    Client::new(nats(subject_prefix).await)
}

async fn run_veritech_server_for_uds_cyclone(
    subject_prefix: String,
) -> JoinHandle<Result<(), ServerError>> {
    tokio::spawn(veritech_server_for_uds_cyclone(subject_prefix).await.run())
}

fn base64_encode(input: impl AsRef<[u8]>) -> String {
    general_purpose::STANDARD_NO_PAD.encode(input)
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test)]
async fn executes_simple_resolver_function() {
    let prefix = nats_prefix();
    run_veritech_server_for_uds_cyclone(prefix.clone()).await;
    let client = client(prefix).await;

    // Not going to check output here--we aren't emitting anything
    let (tx, mut rx) = mpsc::channel(64);
    tokio::spawn(async move {
        while let Some(output) = rx.recv().await {
            info!("output: {:?}", output)
        }
    });

    let request = ResolverFunctionRequest {
        execution_id: "1234".to_string(),
        handler: "numberOfInputs".to_string(),
        component: ResolverFunctionComponent {
            data: ComponentView {
                properties: serde_json::json!({ "foo": "bar", "baz": "quux" }),
                kind: ComponentKind::Standard,
            },
            parents: vec![],
        },
        response_type: ResolverFunctionResponseType::Integer,
        code_base64: base64_encode(
            "function numberOfInputs(input) { return Object.keys(input)?.length ?? 0; }",
        ),
    };

    let result = client
        .execute_resolver_function(tx, &request)
        .await
        .expect("failed to execute resolver function");

    match result {
        FunctionResult::Success(success) => {
            assert_eq!(success.execution_id, "1234");
            assert_eq!(success.data, serde_json::json!(2));
            assert!(!success.unset);
        }
        FunctionResult::Failure(failure) => {
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test)]
async fn type_checks_resolve_function() {
    let prefix = nats_prefix();
    run_veritech_server_for_uds_cyclone(prefix.clone()).await;
    let client = client(prefix).await;

    for response_type in [
        ResolverFunctionResponseType::Array,
        ResolverFunctionResponseType::Integer,
        ResolverFunctionResponseType::Boolean,
        ResolverFunctionResponseType::String,
        ResolverFunctionResponseType::Map,
        ResolverFunctionResponseType::Object,
    ] {
        let value = match response_type {
            ResolverFunctionResponseType::Array => serde_json::json!({ "value": [1, 2, 3, 4] }),
            ResolverFunctionResponseType::Integer => serde_json::json!({ "value": 31337 }),
            ResolverFunctionResponseType::Boolean => serde_json::json!({ "value": true }),
            ResolverFunctionResponseType::String => {
                serde_json::json!({ "value": "a string is a sequence of characters" })
            }
            ResolverFunctionResponseType::Map | ResolverFunctionResponseType::Object => {
                serde_json::json!({ "value": { "an_object": "has keys" } })
            }
            _ => serde_json::json!({ "value": null }),
        };
        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "returnInputValue".to_string(),
            component: ResolverFunctionComponent {
                data: ComponentView {
                    properties: value.clone(),
                    kind: ComponentKind::Standard,
                },
                parents: vec![],
            },
            response_type,
            code_base64: base64_encode("function returnInputValue(input) { return input.value; }"),
        };

        let result = client
            .execute_resolver_function(tx, &request)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "1234");
                if let serde_json::Value::Object(inner) = value {
                    let value = inner.get("value").expect("value should exist").clone();
                    assert_eq!(value, success.data);
                } else {
                    panic!("no value in return data :(")
                }
            }
            FunctionResult::Failure(_) => {
                panic!("should have failed :(");
            }
        }
    }

    for response_type in [
        ResolverFunctionResponseType::Array,
        ResolverFunctionResponseType::Integer,
        ResolverFunctionResponseType::Boolean,
        ResolverFunctionResponseType::String,
        ResolverFunctionResponseType::Map,
        ResolverFunctionResponseType::Object,
    ] {
        let value = match response_type {
            ResolverFunctionResponseType::Array => serde_json::json!({ "value": "foo"}),
            ResolverFunctionResponseType::Integer => serde_json::json!({ "value": "a string" }),
            ResolverFunctionResponseType::Boolean => serde_json::json!({ "value": "a string" }),
            ResolverFunctionResponseType::String => serde_json::json!({ "value": 12345 }),
            ResolverFunctionResponseType::Map | ResolverFunctionResponseType::Object => {
                serde_json::json!({ "value": ["an_object", "has keys" ] })
            }
            _ => serde_json::json!({ "value": null }),
        };
        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "returnInputValue".to_string(),
            component: ResolverFunctionComponent {
                data: ComponentView {
                    properties: value,
                    kind: ComponentKind::Standard,
                },
                parents: vec![],
            },
            response_type: response_type.clone(),
            code_base64: base64_encode("function returnInputValue(input) { return input.value; }"),
        };

        let result = client
            .execute_resolver_function(tx, &request)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                dbg!(success, response_type);
                panic!("should have failed :(");
            }
            FunctionResult::Failure(failure) => {
                assert_eq!(failure.error.kind, "InvalidReturnType");
                assert_eq!(failure.execution_id, "1234");
            }
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test)]
async fn executes_simple_validation() {
    let prefix = nats_prefix();
    run_veritech_server_for_uds_cyclone(prefix.clone()).await;
    let client = client(prefix).await;

    // Not going to check output here--we aren't emitting anything
    let (tx, mut rx) = mpsc::channel(64);
    tokio::spawn(async move {
        while let Some(output) = rx.recv().await {
            info!("output: {:?}", output)
        }
    });

    let request = ValidationRequest {
        execution_id: "31337".to_string(),
        handler: "isThirtyThree".to_string(),
        value: 33.into(),
        code_base64: base64_encode(
            "function isThirtyThree(value) { return { valid: value === 33 }; };",
        ),
    };

    let result = client
        .execute_validation(tx, &request)
        .await
        .expect("failed to execute validation");

    match result {
        FunctionResult::Success(success) => {
            assert_eq!(success.execution_id, "31337");
            assert!(success.valid);
        }
        FunctionResult::Failure(failure) => {
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test)]
async fn executes_simple_schema_variant_definition() {
    let prefix = nats_prefix();
    run_veritech_server_for_uds_cyclone(prefix.clone()).await;
    let client = client(prefix).await;

    // Not going to check output here--we aren't emitting anything
    let (tx, mut rx) = mpsc::channel(64);
    tokio::spawn(async move {
        while let Some(output) = rx.recv().await {
            info!("output: {:?}", output)
        }
    });

    let request = SchemaVariantDefinitionRequest {
        execution_id: "8badf00d".to_string(),
        handler: "asset".to_string(),
        code_base64: base64_encode(
            "function asset() {
                    return {
                        props: [{kind: 'string', name: 'string_prop'}],
                        inputSockets: [], outputSockets: []
                    };
                }",
        ),
    };

    let result = client
        .execute_schema_variant_definition(tx, &request)
        .await
        .expect("failed to execute schema variant definition");

    match result {
        FunctionResult::Success(success) => {
            assert_eq!(success.execution_id, "8badf00d");
            assert_eq!(
                success.definition,
                serde_json::json!({
                    "props": [{
                        "kind": "string",
                        "name": "string_prop",
                    }],
                    "inputSockets": [],
                    "outputSockets": []
                })
            );
        }
        FunctionResult::Failure(failure) => {
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}
