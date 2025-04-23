use std::{
    collections::HashMap,
    env,
};

use base64::{
    Engine,
    engine::general_purpose,
};
use cyclone_core::{
    ActionRunRequest,
    ComponentKind,
    ComponentView,
    ComponentViewWithGeometry,
    FunctionResult,
    FunctionResultFailureErrorKind,
    ManagementRequest,
    ResolverFunctionComponent,
    ResolverFunctionRequest,
    ResolverFunctionResponseType,
    ResourceStatus,
    SchemaVariantDefinitionRequest,
    ValidationRequest,
};
use si_data_nats::{
    NatsClient,
    NatsConfig,
};
use test_log::test;
use tokio::{
    sync::mpsc,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::info;
use uuid::Uuid;
use veritech_client::Client;
use veritech_server::{
    Config,
    CycloneSpec,
    Instance,
    LocalUdsInstance,
    Server,
    StandardConfig,
};

const WORKSPACE_ID: &str = "workspace";
const CHANGE_SET_ID: &str = "changeset";

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

async fn veritech_server_for_uds_cyclone(
    subject_prefix: String,
    shutdown_token: CancellationToken,
) -> Server {
    let mut config_file = veritech_server::ConfigFile::default_local_uds();
    veritech_server::detect_and_configure_development(&mut config_file)
        .expect("failed to determine test configuration");

    let cyclone_spec = CycloneSpec::LocalUds(
        LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to setup cyclone_cmd_path")
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to setup lang_js_cmd_path")
            .all_endpoints()
            .pool_size(4_u32)
            .build()
            .expect("failed to build cyclone spec"),
    );
    let config = Config::builder()
        .nats(nats_config(subject_prefix.clone()))
        .cyclone_spec(cyclone_spec)
        .crypto(config_file.crypto)
        .healthcheck_pool(false)
        .heartbeat_app(false)
        .build()
        .expect("failed to build spec");
    let (server, _disabled_heartbeat_app) = Server::from_config(config, shutdown_token)
        .await
        .expect("failed to create server");
    server
}

async fn client(subject_prefix: String) -> Client {
    Client::new(nats(subject_prefix).await)
}

async fn run_veritech_server_for_uds_cyclone(subject_prefix: String) -> JoinHandle<()> {
    let shutdown_token = CancellationToken::new();
    tokio::spawn(
        veritech_server_for_uds_cyclone(subject_prefix, shutdown_token)
            .await
            .run(),
    )
}

fn base64_encode(input: impl AsRef<[u8]>) -> String {
    general_purpose::STANDARD_NO_PAD.encode(input)
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn executes_simple_management_function() {
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

    let request = ManagementRequest {
        execution_id: "1234".to_string(),
        handler: "numberOfInputs".to_string(),
        current_view: "DEFAULT".to_string(),
        this_component: ComponentViewWithGeometry {
            kind: None,
            properties: serde_json::json!({ "foo": "bar", "baz": "quux", "bar": "foo" }),
            geometry: serde_json::json!({"x": "1", "y": "1"}),
            incoming_connections: serde_json::json!({}),
        },
        components: HashMap::new(),
        variant_socket_map: HashMap::new(),
        code_base64: base64_encode(
            "function numberOfInputs({ thisComponent }) {
                const number = Object.keys(thisComponent.properties)?.length;
                return { status: 'ok', message: `${number}` }
             }",
        ),
        before: vec![],
    };

    let result = client
        .execute_management(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
        .await
        .expect("failed to execute resolver function");

    match result {
        FunctionResult::Success(success) => {
            assert_eq!(Some("3"), success.message.as_deref())
        }
        FunctionResult::Failure(failure) => {
            dbg!("Request details: {:?}", request);
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn executes_simple_action_run() {
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

    let request = ActionRunRequest {
        execution_id: "1234".to_string(),
        handler: "numberOfInputs".to_string(),
        args: serde_json::json!({ "foo": "bar", "baz": "foo" }),
        code_base64: base64_encode(
            "function numberOfInputs(input) { return { status: 'ok', payload: Object.keys(input)?.length ?? 0 } }",
        ),
        before: vec![],
    };

    let result = client
        .execute_action_run(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
        .await
        .expect("failed to execute resolver function");

    match result {
        FunctionResult::Success(success) => {
            dbg!(&success);
            assert_eq!(success.execution_id, "1234");
            assert_eq!(success.payload, Some(serde_json::json!(2)));
            assert_eq!(success.status, ResourceStatus::Ok);
        }
        FunctionResult::Failure(failure) => {
            dbg!("Request details: {:?}", request);
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
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
        before: vec![],
    };

    let result = client
        .execute_resolver_function(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
        .await
        .expect("failed to execute resolver function");

    match result {
        FunctionResult::Success(success) => {
            assert_eq!(success.execution_id, "1234");
            assert_eq!(success.data, serde_json::json!(2));
            assert!(!success.unset);
        }
        FunctionResult::Failure(failure) => {
            dbg!("Request details: {:?}", request);
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
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

        let execution_id = "type_checks_resolve_function";

        let request = ResolverFunctionRequest {
            execution_id: execution_id.to_string(),
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
            before: vec![],
        };

        let result = client
            .execute_resolver_function(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, execution_id.to_string());
                if let serde_json::Value::Object(inner) = value {
                    let value = inner.get("value").expect("value should exist").clone();
                    assert_eq!(value, success.data);
                } else {
                    dbg!("Request details: {:?}", request);
                    panic!("no value in return data :(")
                }
            }
            FunctionResult::Failure(_) => {
                dbg!("Request details: {:?}", request);
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

        let execution_id = "type_checks_resolve_function";
        let request = ResolverFunctionRequest {
            execution_id: execution_id.to_string(),
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
            before: vec![],
        };

        let result = client
            .execute_resolver_function(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                dbg!(success, response_type);
                panic!("should have failed :(");
            }
            FunctionResult::Failure(failure) => {
                assert_eq!(
                    failure.error().kind,
                    FunctionResultFailureErrorKind::InvalidReturnType
                );
                assert_eq!(failure.execution_id(), execution_id);
            }
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
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
        handler: "".to_string(),
        value: Some(33.into()),
        validation_format: r#"{"type":"number","flags":{"presence":"required"},"rules":[{"name":"integer"},{"name":"min","args":{"limit":33}},{"name":"max","args":{"limit":33}}]}"#.to_string(),
        code_base64: "".to_string(),
        before: vec![],
    };

    let result = client
        .execute_validation(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
        .await
        .expect("failed to execute validation");

    match result {
        FunctionResult::Success(success) => {
            assert_eq!(success.execution_id, "31337");
            assert!(success.error.is_none());
        }
        FunctionResult::Failure(failure) => {
            dbg!("Request details: {:?}", request);
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
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
        .execute_schema_variant_definition(tx, &request, WORKSPACE_ID, CHANGE_SET_ID)
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
            dbg!("Request details: {:?}", request);
            panic!("function did not succeed and should have: {failure:?}")
        }
    }
}
