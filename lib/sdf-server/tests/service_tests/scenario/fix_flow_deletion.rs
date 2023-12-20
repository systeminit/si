use axum::Router;
use dal::FixCompletionStatus;
use dal_test::{sdf_test, AuthToken, DalContextHead};
use pretty_assertions_sorted::assert_eq;

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

    let actions = harness.find_change_set(&ctx).await.actions;

    let expected = vec![
        (derke.component_id, "create"),
        (alfajer.component_id, "create"),
        (boaster.component_id, "create"),
        (chronicle.component_id, "create"),
        (leo.component_id, "create"),
    ];
    assert_eq!(
        expected, // expected
        actions
            .values()
            .map(|r| (r.component_id, r.name.as_str()))
            .collect::<Vec<_>>(), // actual
    );

    let fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    assert!(fix_batch_history_views.is_empty());

    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Check that the fix batch succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    let create_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert!(fix_batch_history_views.is_empty());
    assert_eq!(
        FixCompletionStatus::Success,                 // expected
        create_view.status.expect("no status found")  // actual
    );

    // Go back to model, immediately merge and come back. We should still see no recommended action
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop3")
        .await;

    let actions = harness.find_change_set(&ctx).await.actions;
    assert!(actions.is_empty());

    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Ensure the resource exists after creation.
    let diagram = harness.get_diagram(ctx.visibility()).await;
    for component in diagram.components() {
        assert_eq!(
            true,                     // expected
            component.has_resource()  // actual
        );
    }

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
        assert_eq!(
            true,                     // expected
            component.has_resource()  // actual
        );
    }

    // Now, delete all the components and come back. The order at which we delete should have no
    // effect on the order of the actions
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

    let actions = harness.find_change_set(&ctx).await.actions;

    let expected = vec![
        (leo.component_id, "delete"),
        (chronicle.component_id, "delete"),
        (boaster.component_id, "delete"),
        (alfajer.component_id, "delete"),
        (derke.component_id, "delete"),
    ];
    assert_eq!(
        expected, // expected
        actions
            .values()
            .map(|r| (r.component_id, r.name.as_str()))
            .collect::<Vec<_>>(), // actual
    );

    let fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    assert!(fix_batch_history_views.is_empty());

    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Check that the fix batch succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    assert_eq!(
        2,                             // expected
        fix_batch_history_views.len()  // actual
    );
    let destroy_view = fix_batch_history_views
        .pop()
        .expect("found empty batch history views");

    assert_eq!(
        FixCompletionStatus::Success,                  // expected
        destroy_view.status.expect("no status found")  // actual
    );

    // Go back to model, immediately merge and come back!
    harness
        .create_change_set_and_update_ctx(&mut ctx, "poop5")
        .await;

    let actions = harness.find_change_set(&ctx).await.actions;
    assert!(actions.is_empty());

    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // TODO(nick): mix in creation and deletion actions as well as scenarios where not
    // all fixes are ran all at once.
}
