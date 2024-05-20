use dal::{
    func::backend::js_action::DeprecatedActionRunResult, ActionCompletionStatus, Component,
    DalContext, DeprecatedActionBatch, DeprecatedActionPrototype, DeprecatedActionRunner,
};
use dal_test::helpers::create_component_for_schema_name;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn get_by_id(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        DeprecatedActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner"),
        runner
    );
}

#[test]
async fn set_resource(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.resource, None);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let resource = DeprecatedActionRunResult {
        status: None,
        payload: None,
        message: None,
        last_synced: None,
    };
    runner
        .set_resource(ctx, Some(resource.clone()))
        .await
        .expect("unable to set completion status");
    assert_eq!(runner.resource.as_ref(), Some(&resource));

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        DeprecatedActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner")
            .resource,
        Some(resource)
    );
}

#[test]
async fn set_completion_message(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.completion_message, None);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    runner
        .set_completion_message(ctx, Some("my-message".to_owned()))
        .await
        .expect("unable to set completion message");
    assert_eq!(runner.completion_message, Some("my-message".to_owned()));

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        DeprecatedActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner")
            .completion_message,
        Some("my-message".to_owned())
    );
}

#[test]
async fn set_completion_status(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.completion_status, None);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    runner
        .set_completion_status(ctx, Some(ActionCompletionStatus::Success))
        .await
        .expect("unable to set completion status");
    assert_eq!(
        runner.completion_status,
        Some(ActionCompletionStatus::Success)
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        DeprecatedActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner")
            .completion_status,
        Some(ActionCompletionStatus::Success)
    );
}

#[test]
async fn set_started_at(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.started_at, None);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    runner
        .set_started_at(ctx)
        .await
        .expect("unable to set completion status");
    assert!(runner.started_at.is_some());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(DeprecatedActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner")
        .started_at
        .is_some());
}

#[test]
async fn set_finished_at(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert!(runner.finished_at.is_none());
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    runner
        .set_finished_at(ctx)
        .await
        .expect("unable to set completion status");
    assert!(runner.finished_at.is_some());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(DeprecatedActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner")
        .finished_at
        .is_some());
}

#[test]
async fn stamp_started(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert!(runner.started_at.is_none());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    runner
        .stamp_started(ctx)
        .await
        .expect("unable to set stamp started");
    assert!(runner.started_at.is_some());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(DeprecatedActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner")
        .started_at
        .is_some());

    assert!(runner.stamp_started(ctx).await.is_err());
}

#[test]
async fn stamp_finished(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert!(runner.started_at.is_none());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(runner
        .stamp_finished(
            ctx,
            ActionCompletionStatus::Success,
            Some("message".to_owned()),
            Some(DeprecatedActionRunResult {
                status: None,
                payload: None,
                message: None,
                last_synced: None
            })
        )
        .await
        .is_err());
    runner
        .stamp_started(ctx)
        .await
        .expect("unable to set stamp started");
    runner
        .stamp_finished(
            ctx,
            ActionCompletionStatus::Success,
            Some("message".to_owned()),
            Some(DeprecatedActionRunResult {
                status: None,
                payload: None,
                message: None,
                last_synced: None,
            }),
        )
        .await
        .expect("unable to set stamp finished");
    assert!(runner.finished_at.is_some());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let runner = DeprecatedActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner");
    assert!(runner.finished_at.is_some());
    assert_eq!(
        runner.completion_status,
        Some(ActionCompletionStatus::Success)
    );
}

#[test]
async fn for_batch(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let runners = DeprecatedActionRunner::for_batch(ctx, batch.id)
        .await
        .expect("unable to get action runners for batch");
    assert_eq!(runners.len(), 1);
    assert_eq!(runners[0], runner);
}

#[test]
async fn run(ctx: &mut DalContext) {
    let batch = DeprecatedActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        DeprecatedActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(runner.run(ctx).await.expect("unable to run").0.is_some());
}
