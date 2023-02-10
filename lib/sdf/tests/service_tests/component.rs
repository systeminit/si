use dal::{StandardModel, Visibility};
use dal_test::{
    test,
    test_harness::{create_component_for_schema_variant, create_schema, create_schema_variant},
};
use sdf::service::component::get_components_metadata::{
    GetComponentsMetadataRequest, GetComponentsMetadataResponse,
};

use crate::{service_tests::api_request_auth_query, test_setup};

#[test]
async fn get_components_metadata() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
        app,
        _nba,
        auth_token,
        dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );
    let visibility = Visibility::new_head(false);

    let schema = create_schema(&dal_ctx).await;

    let mut schema_variant = create_schema_variant(&dal_ctx, *schema.id()).await;
    schema_variant
        .finalize(&dal_ctx, None)
        .await
        .expect("could not finalize schema variant");

    let _component = create_component_for_schema_variant(&dal_ctx, schema_variant.id()).await;

    dal_ctx.commit().await.expect("cannot commit transaction");

    let request = GetComponentsMetadataRequest { visibility };
    let response: GetComponentsMetadataResponse = api_request_auth_query(
        app,
        "/api/component/get_components_metadata",
        &auth_token,
        &request,
    )
    .await;

    assert_eq!(response.data[0].schema_name, schema.name());
}
