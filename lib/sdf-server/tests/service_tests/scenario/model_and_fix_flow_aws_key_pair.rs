use axum::Router;
use dal::FixCompletionStatus;
use dal_test::{sdf_test, AuthToken, DalContextHead};
use pretty_assertions_sorted::assert_eq;
use sdf_server::service::fix::run::FixRunRequest;

use crate::service_tests::scenario::ScenarioHarness;

/// This test runs through the entire model flow and fix flow lifecycle for solely an AWS Key Pair.
///
/// It is recommended to run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=aws region,aws keypair
/// ```
#[sdf_test]
#[ignore]
async fn model_and_fix_flow_aws_key_pair(
    DalContextHead(mut ctx): DalContextHead,
    app: Router,
    AuthToken(auth_token): AuthToken,
) {
    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(&ctx, app, auth_token, &["Region", "Key Pair"]).await;

    // Enter a new change set. We will not go through the routes for this.
    harness
        .create_change_set_and_update_ctx(&mut ctx, "swans")
        .await;

    // Create all AWS components.
    let region = harness.create_node(&ctx, "Region", None).await;
    let key_pair = harness
        .create_node(&ctx, "Key Pair", Some(region.node_id))
        .await;

    // Update property editor values.
    harness
        .update_value(
            &ctx,
            key_pair.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-key"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            key_pair.component_id,
            &["domain", "KeyType"],
            Some(serde_json::json!["rsa"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            region.component_id,
            &["domain", "region"],
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // Ensure everything looks as expected.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-key",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateAwsKeyPairJSON": {
                    "code": "{\n\t\"KeyName\": \"toddhoward-key\",\n\t\"KeyType\": \"rsa\",\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"key-pair\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"toddhoward-key\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": "toddhoward-key",
                },
                "region": "us-east-2",
                "KeyName": "toddhoward-key",
                "KeyType": "rsa",
                "awsResourceType": "key-pair",
            },
            "qualification": {
                "si:qualificationKeyPairCanCreate": {
                    "result": "success",
                    "message": "component qualified",
                },
            },
        }], // expected
        key_pair
            .view(&ctx)
            .await
            .drop_confirmation()
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "type": "configurationFrame",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2",
            },
        }], // expected
        region
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );

    // Apply the change set and get rolling!
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Check the confirmations and ensure they look as we expect.
    let (confirmations, mut recommendations) = harness.list_confirmations(&mut ctx).await;
    assert_eq!(
        1,                   // expected
        confirmations.len()  // actual
    );
    let recommendation = recommendations.pop().expect("no recommendations found");
    assert!(recommendations.is_empty());

    // Run the fix for the confirmation.
    let fix_batch_id = harness
        .run_fixes(
            &mut ctx,
            vec![FixRunRequest {
                attribute_value_id: recommendation.confirmation_attribute_value_id,
                component_id: recommendation.component_id,
                action_name: recommendation.recommended_action,
            }],
        )
        .await;

    // Check that the fix succeeded.
    let mut fix_batch_history_views = harness.list_fixes(&mut ctx).await;
    let fix_batch_history_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert!(fix_batch_history_views.is_empty());
    assert_eq!(
        fix_batch_id,              // expected
        fix_batch_history_view.id, // actual
    );
    assert_eq!(
        Some(FixCompletionStatus::Success), // expected
        fix_batch_history_view.status
    );
}
