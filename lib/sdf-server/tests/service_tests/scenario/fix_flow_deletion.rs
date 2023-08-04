use std::collections::HashSet;

use axum::Router;
use dal::{
    component::confirmation::view::{ConfirmationStatus, RecommendationView},
    ActionKind, ComponentId, FixCompletionStatus,
};
use dal_test::{sdf_test, AuthToken, DalContextHead};
use pretty_assertions_sorted::assert_eq;
use sdf_server::service::{
    dev::{CREATE_CONFIRMATION_NAME, DELETE_CONFIRMATION_NAME},
    fix::run::FixRunRequest,
};

use crate::service_tests::scenario::ScenarioHarness;

/// Recommendation: run this test with the following environment variable...
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[sdf_test]
#[ignore]
async fn fix_flow_deletion(
    DalContextHead(mut ctx): DalContextHead,
    app: Router,
    AuthToken(auth_token): AuthToken,
) {
    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(&ctx, app, auth_token, &[]).await;

    // Author a schema using the appropriate route. We'll add it to our harness' cache afterwards.
    // We'll do this all in a changeset and then apply it.
    assert!(ctx.visibility().is_head());
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop1")
        .await;
    let schema_name = "lock//in";
    harness
        .author_single_schema_with_default_variant(ctx.visibility(), schema_name)
        .await;
    harness.add_schemas(&ctx, &[schema_name]).await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // In a new changeset, create nodes and connections. Apply the changeset when finished.
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop2")
        .await;
    let boaster = harness
        .create_node(ctx.visibility(), schema_name, None)
        .await;
    let derke = harness
        .create_node(ctx.visibility(), schema_name, None)
        .await;
    let alfajer = harness
        .create_node(ctx.visibility(), schema_name, None)
        .await;
    let leo = harness
        .create_node(ctx.visibility(), schema_name, None)
        .await;
    let chronicle = harness
        .create_node(ctx.visibility(), schema_name, None)
        .await;
    harness
        .create_connection(&ctx, alfajer.node_id, boaster.node_id, "universal")
        .await;
    harness
        .create_connection(&ctx, boaster.node_id, leo.node_id, "universal")
        .await;
    harness
        .create_connection(&ctx, alfajer.node_id, chronicle.node_id, "universal")
        .await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // On HEAD, check the confirmations and recommendations to see that they match what we expect.
    // We also want to ensure that the recommendations are topologically sorted with stable
    // ordering (i.e. use a "Vec" with non-arbitrary ordering for the assertion(s)).
    let (confirmations, recommendations) = harness.list_confirmations(ctx.visibility()).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    let recommendation_metadata = recommendations
        .iter()
        .map(|r| r.into())
        .collect::<Vec<RecommendationMetadata>>();

    let expected = vec![
        RecommendationMetadata {
            component_id: derke.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: alfajer.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: boaster.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: chronicle.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: leo.component_id,
            action_kind: ActionKind::Create,
        },
    ];
    assert_eq!(
        expected,                // expected
        recommendation_metadata  // actual
    );

    // As we are checking that the confirmations look as we expect, assemble fix run requests.
    let mut seen_component_ids = HashSet::new();
    for confirmation in confirmations {
        seen_component_ids.insert(confirmation.component_id);

        if confirmation.title == DELETE_CONFIRMATION_NAME {
            assert_eq!(
                ConfirmationStatus::Success, // expected
                confirmation.status          // actual
            );
        } else if confirmation.title == CREATE_CONFIRMATION_NAME {
            assert_eq!(
                ConfirmationStatus::Failure, // expected
                confirmation.status          // actual
            );
        } else {
            panic!("could not find anticipated confirmation title to determine assertions (found confirmation title: {})", confirmation.title);
        };
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );

    // Run the fixes for the corresponding confirmations. We will use the exact order of the
    // recommendations during assembly.
    let mut fix_requests = Vec::new();
    for recommendation in recommendations {
        assert_eq!(
            ActionKind::Create,         // expected
            recommendation.action_kind  // actual
        );
        fix_requests.push(FixRunRequest {
            attribute_value_id: recommendation.confirmation_attribute_value_id,
            component_id: recommendation.component_id,
            action_prototype_id: recommendation.action_prototype_id,
        });
    }
    assert_eq!(
        5,                  // expected
        fix_requests.len()  // actual
    );
    let create_fix_batch_id = harness.run_fixes(ctx.visibility(), fix_requests).await;

    // Check that the fix batch succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    let create_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert!(fix_batch_history_views.is_empty());
    assert_eq!(
        create_fix_batch_id, // expected
        create_view.id,      // actual
    );
    assert_eq!(
        FixCompletionStatus::Success,                 // expected
        create_view.status.expect("no status found")  // actual
    );

    // Check confirmations again on HEAD. We should have no recommendations this time.
    let (confirmations, recommendations) = harness.list_confirmations(ctx.visibility()).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    assert!(recommendations.is_empty());
    let mut seen_component_ids = HashSet::new();
    for confirmation in confirmations {
        seen_component_ids.insert(confirmation.component_id);
        assert_eq!(
            ConfirmationStatus::Success, // expected
            confirmation.status          // actual
        );
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );

    // Go back to model, immediately merge and come back. We should still see no recommendations.
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop3")
        .await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;
    let (confirmations, recommendations) = harness.list_confirmations(ctx.visibility()).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    assert!(recommendations.is_empty());
    let mut seen_component_ids = HashSet::new();
    for confirmation in confirmations {
        seen_component_ids.insert(confirmation.component_id);
        assert_eq!(
            ConfirmationStatus::Success, // expected
            confirmation.status          // actual
        );
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );

    // Ensure the resource exists after creation.
    let diagram = harness.get_diagram(ctx.visibility()).await;
    for component in diagram.components() {
        let maybe_data_raw = component.resource().data.clone().expect("data is empty");
        let data = serde_json::to_string(&maybe_data_raw).expect("could not deserialize data");
        assert_eq!(
            "\"poop\"", // expected
            &data       // actual
        );
    }

    // Refresh the resources. The order at which we refresh should have no
    // effect on the order of the confirmations and recommendations that come back.
    assert!(ctx.visibility().is_head());
    harness
        .resource_refresh(ctx.visibility(), boaster.component_id)
        .await;
    harness
        .resource_refresh(ctx.visibility(), derke.component_id)
        .await;
    harness
        .resource_refresh(ctx.visibility(), leo.component_id)
        .await;
    harness
        .resource_refresh(ctx.visibility(), alfajer.component_id)
        .await;
    harness
        .resource_refresh(ctx.visibility(), chronicle.component_id)
        .await;

    // Ensure the resource continues to exist after refresh.
    let diagram = harness.get_diagram(ctx.visibility()).await;
    for component in diagram.components() {
        let maybe_data_raw = component.resource().data.clone().expect("data is empty");
        let data = serde_json::to_string(&maybe_data_raw).expect("could not deserialize data");
        assert_eq!(
            "\"poop\"", // expected
            &data       // actual
        );
    }

    // Now, delete all the components and come back. The order at which we delete should have no
    // effect on the order of the confirmations and recommendations that come back.
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop4")
        .await;
    harness
        .delete_component(ctx.visibility(), leo.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), boaster.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), chronicle.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), derke.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), alfajer.component_id)
        .await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Once the change set is applied, check the confirmations. We should now have destroy
    // recommendations.
    let (confirmations, recommendations) = harness.list_confirmations(ctx.visibility()).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    let recommendation_metadata = recommendations
        .iter()
        .map(|r| r.into())
        .collect::<Vec<RecommendationMetadata>>();

    let expected = vec![
        RecommendationMetadata {
            component_id: leo.component_id,
            action_kind: ActionKind::Delete,
        },
        RecommendationMetadata {
            component_id: chronicle.component_id,
            action_kind: ActionKind::Delete,
        },
        RecommendationMetadata {
            component_id: boaster.component_id,
            action_kind: ActionKind::Delete,
        },
        RecommendationMetadata {
            component_id: alfajer.component_id,
            action_kind: ActionKind::Delete,
        },
        RecommendationMetadata {
            component_id: derke.component_id,
            action_kind: ActionKind::Delete,
        },
    ];
    assert_eq!(
        expected,                // expected
        recommendation_metadata  // actual
    );

    let mut seen_component_ids = HashSet::new();
    for confirmation in confirmations {
        seen_component_ids.insert(confirmation.component_id);

        if confirmation.title == DELETE_CONFIRMATION_NAME {
            assert_eq!(
                ConfirmationStatus::Failure, // expected
                confirmation.status          // actual
            );
        } else if confirmation.title == CREATE_CONFIRMATION_NAME {
            assert_eq!(
                ConfirmationStatus::Success, // expected
                confirmation.status          // actual
            );
        } else {
            panic!("could not find anticipated confirmation title to determine assertions (found confirmation title: {})", confirmation.title);
        };
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );

    // Run the fixes for the corresponding confirmations. We will use the exact order of the
    // recommendations during assembly.
    let mut fix_requests = Vec::new();
    for recommendation in recommendations {
        assert_eq!(
            ActionKind::Delete,         // expected
            recommendation.action_kind  // actual
        );
        fix_requests.push(FixRunRequest {
            attribute_value_id: recommendation.confirmation_attribute_value_id,
            component_id: recommendation.component_id,
            action_prototype_id: recommendation.action_prototype_id,
        });
    }
    assert_eq!(
        5,                  // expected
        fix_requests.len()  // actual
    );
    let destroy_fix_batch_id = harness.run_fixes(ctx.visibility(), fix_requests).await;

    // Check that the fix batch succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    assert_eq!(
        2,                             // expected
        fix_batch_history_views.len()  // actual
    );
    let first_view = fix_batch_history_views
        .pop()
        .expect("found empty batch history views");
    let second_view = fix_batch_history_views
        .pop()
        .expect("found empty batch history views");

    // Find the destroy view and ignore the create view.
    let mut destroy_view = None;
    if destroy_fix_batch_id == first_view.id {
        destroy_view = Some(first_view);
        assert_eq!(
            create_fix_batch_id, // expected
            second_view.id       // actual
        );
    } else if destroy_fix_batch_id == second_view.id {
        destroy_view = Some(second_view);
        assert_eq!(
            create_fix_batch_id, // expected
            first_view.id        // actual
        );
    }
    let destroy_view = destroy_view.expect("batch history view not found");

    // Ensure the destroy view succeeded.
    assert_eq!(
        destroy_fix_batch_id, // expected
        destroy_view.id,      // actual
    );
    assert_eq!(
        FixCompletionStatus::Success,                  // expected
        destroy_view.status.expect("no status found")  // actual
    );

    // Check confirmations on HEAD.
    let (confirmations, recommendations) = harness.list_confirmations(ctx.visibility()).await;
    assert!(confirmations.is_empty());
    assert!(recommendations.is_empty());

    // Go back to model, immediately merge and come back!
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop5")
        .await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;
    let (confirmations, recommendations) = harness.list_confirmations(ctx.visibility()).await;
    assert!(confirmations.is_empty());
    assert!(recommendations.is_empty());

    // TODO(nick): mix in creation and deletion recommendations as well as scenarios where not
    // all fixes are ran all at once.
}

#[derive(Debug, Eq, PartialEq)]
struct RecommendationMetadata {
    component_id: ComponentId,
    action_kind: ActionKind,
}

#[allow(clippy::from_over_into)]
impl Into<RecommendationMetadata> for &RecommendationView {
    fn into(self) -> RecommendationMetadata {
        RecommendationMetadata {
            component_id: self.component_id,
            action_kind: self.action_kind,
        }
    }
}
