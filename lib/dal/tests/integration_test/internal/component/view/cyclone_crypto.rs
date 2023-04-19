use base64::{engine::general_purpose, Engine};
use dal::DalContext;

use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use tokio::sync::mpsc;
use veritech_client::ResolverFunctionResponseType;

#[test]
async fn cyclone_crypto_e2e(ctx: &DalContext) {
    let (tx, _rx) = mpsc::channel(64);
    let secret_value = "Beware Cuca will catch you";
    let secret = serde_json::to_string(&serde_json::json!({
        "key": secret_value,
    }))
    .expect("Secret serialization failed");
    let encoded = ctx.encryption_key().encrypt_and_encode(&secret);
    let code = format!(
        "function testE2ECrypto(input) {{ return input.secret.message.key === '{secret_value}'; }}"
    );
    let request = veritech_client::ResolverFunctionRequest {
        execution_id: "seujorge".to_owned(),
        handler: "testE2ECrypto".to_owned(),
        component: veritech_client::ResolverFunctionComponent {
            data: veritech_client::ComponentView {
                kind: veritech_client::ComponentKind::Credential,
                properties: serde_json::json!({
                    "secret": {
                        "name": "ufo",
                        "secret_kind": "dockerHub",
                        "object_type": "credential",
                        "message": {
                            "cycloneEncryptedDataMarker": true,
                            "encryptedSecret": encoded,
                        },
                    },
                }),
            },
            parents: Vec::new(),
        },
        response_type: ResolverFunctionResponseType::Boolean,
        code_base64: general_purpose::STANDARD_NO_PAD.encode(&code),
    };
    let result = ctx
        .veritech()
        .execute_resolver_function(tx, &request)
        .await
        .expect("Veritech run failed");
    match result {
        veritech_client::FunctionResult::Success(result) => {
            assert_eq!(result.data, serde_json::Value::Bool(true))
        }
        veritech_client::FunctionResult::Failure(err) => panic!("Veritech run failed: {err:?}"),
    }
}
