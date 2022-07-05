use crate::dal::test;
use dal::{
    test_harness::encrypt_message, EncryptedSecret, SecretAlgorithm, SecretKind, SecretObjectType,
    SecretVersion, StandardModel, Visibility,
};
use hyper::Method;
use sdf::service::secret::create_secret::{CreateSecretRequest, CreateSecretResponse};

use crate::{service_tests::api_request_auth_json_body, test_setup};

#[test]
async fn create_secret() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        _veritech,
        _encr_key,
        app,
        nba,
        auth_token,
        dal_ctx,
        dal_txns,
        _faktory,
    );

    let visibility = Visibility::new_head(false);

    let message = serde_json::json!({"artist":"Billy Talent"});
    let crypted = encrypt_message(&dal_ctx, *nba.key_pair.id(), &message).await;

    let request = CreateSecretRequest {
        name: "reckless-paradise".to_string(),
        object_type: SecretObjectType::Credential,
        kind: SecretKind::DockerHub,
        crypted,
        key_pair_id: *nba.key_pair.id(),
        version: SecretVersion::V1,
        algorithm: SecretAlgorithm::Sealedbox,
        workspace_id: *nba.workspace.id(),
        visibility,
    };

    let response: CreateSecretResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/secret/create_secret",
        &auth_token,
        &request,
    )
    .await;
    assert_eq!(response.secret.name(), "reckless-paradise");
    assert_eq!(response.secret.object_type(), &SecretObjectType::Credential);
    assert_eq!(response.secret.kind(), &SecretKind::DockerHub);

    let decrypted_secret = EncryptedSecret::get_by_id(&dal_ctx, response.secret.id())
        .await
        .expect("failed to fetch encrypted secret")
        .expect("failed to find encrypted secret in tenancy and/or visibility")
        .decrypt(&dal_ctx)
        .await
        .expect("failed to decrypt secret");

    assert_eq!(decrypted_secret.name(), "reckless-paradise");
    assert_eq!(decrypted_secret.object_type(), SecretObjectType::Credential);
    assert_eq!(decrypted_secret.kind(), SecretKind::DockerHub);
    // We don't provide a direct getter for the raw decrypted message (higher effort should mean
    // less chance of developer error when handling `DecryptedSecret` types), so we'll serialize to
    // a `Value` to compare messages
    let decrypted_value =
        serde_json::to_value(&decrypted_secret).expect("failed to serial decrypted into Value");
    assert_eq!(decrypted_value["message"], message);
}
