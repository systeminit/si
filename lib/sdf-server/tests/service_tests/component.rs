use axum::Router;
use dal::{StandardModel, Visibility};
use dal_test::{
    sdf_test,
    test_harness::{create_component_for_schema_variant, create_schema, create_schema_variant},
    AuthTokenRef, DalContextHead,
};
use sdf_server::service::component::get_components_metadata::{
    GetComponentsMetadataRequest, GetComponentsMetadataResponse,
};

use crate::service_tests::api_request_auth_query;

#[sdf_test]
async fn get_components_metadata(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
) {
    let visibility = Visibility::new_head(false);
    let schema = create_schema(&ctx).await;
    let mut schema_variant = create_schema_variant(&ctx, *schema.id()).await;
    schema_variant
        .finalize(&ctx, None)
        .await
        .expect("could not finalize schema variant");

    let _component = create_component_for_schema_variant(&ctx, schema_variant.id()).await;
    ctx.commit().await.expect("cannot commit transaction");

    let request = GetComponentsMetadataRequest { visibility };

    let response: GetComponentsMetadataResponse = api_request_auth_query(
        app,
        "/api/component/get_components_metadata",
        auth_token,
        &request,
    )
    .await;

    assert_eq!(response.data[0].schema_name, schema.name());
}
