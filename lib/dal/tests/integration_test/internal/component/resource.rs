use dal::func::backend::js_action::ActionRunResult;
use dal::{ChangeSet, DalContext, ResourceView};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=test
/// ```
#[test]
async fn get_resource(mut octx: DalContext) {
    let ctx = &mut octx;

    let mut bagger = ComponentBagger::new();
    let fallout_bag = bagger.create_component(ctx, "fallout", "fallout").await;
    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Check the resources after creation.
    let mut expected = ResourceView {
        status: Some(ResourceStatus::Ok),
        message: None,
        data: None,
        logs: vec![],
        last_synced: None,
    };
    let actual = ResourceView::get_by_component_id(ctx, &fallout_bag.component_id)
        .await
        .expect("could not get resource view");
    assert_eq!(
        expected, // expected
        actual,   // actual
    );

    // "Create" a resource.
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not fetch change set by pk")
        .expect("no change set found for pk");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");
    let fallout_component = fallout_bag.component(ctx).await;
    fallout_component
        .set_resource(
            ctx,
            ActionRunResult {
                status: Some(ResourceStatus::Ok),
                payload: Some(serde_json::json![{ "poop": true }]),
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
        )
        .await
        .expect("could not set resource");
    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Check the resources again.
    expected.data = Some(serde_json::json![{ "poop": true}]);
    let actual = ResourceView::get_by_component_id(ctx, &fallout_bag.component_id)
        .await
        .expect("could not get resource view");
    assert_eq!(
        expected, // expected
        actual,   // actual
    );

    // "Delete" the created resource.
    fallout_component
        .set_resource(
            ctx,
            ActionRunResult {
                status: Some(ResourceStatus::Ok),
                payload: None,
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
        )
        .await
        .expect("could not set resource");
    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Check the resources again.
    expected.data = None;
    let actual = ResourceView::get_by_component_id(ctx, &fallout_bag.component_id)
        .await
        .expect("could not get resource view");
    assert_eq!(
        expected, // expected
        actual,   // actual
    );
}
