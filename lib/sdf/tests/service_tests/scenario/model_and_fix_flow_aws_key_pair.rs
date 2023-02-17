use dal::{DalContext, FixCompletionStatus};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use sdf::service::fix::run::FixRunRequest;

use crate::service_tests::scenario::ScenarioHarness;
use crate::test_setup;

/// This test runs through the entire model flow and fix flow lifecycle for solely an AWS Key Pair.
///
/// It is recommended to run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=awsRegion,awsKeyPair
/// ```
#[test]
#[ignore]
async fn model_and_fix_flow_aws_key_pair() {
    test_setup!(
        _sdf_ctx,
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
        ctx,
        _job_processor,
        _council_subject_prefix,
    );
    // Just borrow it the whole time because old habits die hard.
    let ctx: &mut DalContext = &mut ctx;

    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(ctx, app, auth_token, &["Region", "Key Pair"]).await;

    // Enter a new change set. We will not go through the routes for this.
    harness.create_change_set_and_update_ctx(ctx, "swans").await;

    // Create all AWS components.
    let region = harness.create_node(ctx, "Region", None).await;
    let key_pair = harness
        .create_node(ctx, "Key Pair", Some(region.node_id))
        .await;

    // Update property editor values.
    harness
        .update_value(
            ctx,
            key_pair.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-key"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            key_pair.component_id,
            &["domain", "KeyType"],
            Some(serde_json::json!["rsa"]),
        )
        .await;
    harness
        .update_value(
            ctx,
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
        key_pair.view(ctx).await.drop_confirmation().to_value(), // actual
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
        region.view(ctx).await.to_value(), // actual
    );

    // Apply the change set and get rolling!
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(ctx)
        .await;

    // Check the confirmations and ensure they look as we expect.
    let mut confirmations = harness.list_confirmations(ctx).await;
    let mut confirmation = confirmations.pop().expect("no confirmations found");
    assert!(confirmations.is_empty());
    let recommendation = confirmation
        .recommendations
        .pop()
        .expect("no recommendations found");
    assert!(confirmation.recommendations.is_empty());

    // Run the fix for the confirmation.
    let fix_batch_id = harness
        .run_fixes(
            ctx,
            vec![FixRunRequest {
                attribute_value_id: recommendation.confirmation_attribute_value_id,
                component_id: recommendation.component_id,
                action_name: recommendation.recommended_action,
            }],
        )
        .await;

    // Check that the fix succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx).await;
    let fix_batch_history_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert!(fix_batch_history_views.is_empty());
    assert_eq!(
        fix_batch_id,              // expected
        fix_batch_history_view.id, // actual
    );
    assert_eq!(
        FixCompletionStatus::Success, // expected
        fix_batch_history_view.status
    );
}
