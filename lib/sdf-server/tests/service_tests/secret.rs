use axum::Router;
use dal::{
    EncryptedSecret, SecretAlgorithm, SecretKind, SecretObjectType, SecretVersion, StandardModel,
    Visibility, WorkspaceSignup,
};
use dal_test::{sdf_test, test_harness::encrypt_message, AuthTokenRef, DalContextHead};
use hyper::Method;
use sdf_server::service::secret::create_secret::{CreateSecretRequest, CreateSecretResponse};

use crate::service_tests::api_request_auth_json_body;

#[sdf_test]
async fn create_secret(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
    nw: WorkspaceSignup,
) {
    let visibility = Visibility::new_head(false);

    let message = serde_json::json!({"artist":"Billy Talent"});
    let crypted = encrypt_message(&ctx, nw.key_pair.pk(), &message).await;

    let request = CreateSecretRequest {
        name: "reckless-paradise".to_string(),
        object_type: SecretObjectType::Credential,
        kind: SecretKind::DockerHub,
        crypted,
        key_pair_pk: nw.key_pair.pk(),
        version: SecretVersion::V1,
        algorithm: SecretAlgorithm::Sealedbox,
        visibility,
    };

    let response: CreateSecretResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/secret/create_secret",
        auth_token,
        &request,
    )
    .await;
    assert_eq!(response.secret.name(), "reckless-paradise");
    assert_eq!(response.secret.object_type(), &SecretObjectType::Credential);
    assert_eq!(response.secret.kind(), &SecretKind::DockerHub);

    let decrypted_secret = EncryptedSecret::get_by_id(&ctx, response.secret.id())
        .await
        .expect("failed to fetch encrypted secret")
        .expect("failed to find encrypted secret in tenancy and/or visibility")
        .decrypt(&ctx)
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
