use dal::{
    func::backend::js_action::ActionRunResult, ActionBatch, ActionCompletionStatus,
    ActionPrototype, ActionRunner, Component, DalContext,
};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn get_by_id(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let runner = ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
        .await
        .expect("unable to create action runner");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        ActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner"),
        runner
    );
}

#[test]
async fn set_resource(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.resource, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let resource = ActionRunResult {
        status: None,
        payload: None,
        message: None,
        logs: Vec::new(),
        last_synced: None,
    };
    runner
        .set_resource(ctx, Some(resource.clone()))
        .await
        .expect("unable to set completion status");
    assert_eq!(runner.resource.as_ref(), Some(&resource));

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        ActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner")
            .resource,
        Some(resource)
    );
}

#[test]
async fn set_completion_message(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.completion_message, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    runner
        .set_completion_message(ctx, Some("my-message".to_owned()))
        .await
        .expect("unable to set completion message");
    assert_eq!(runner.completion_message, Some("my-message".to_owned()));

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        ActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner")
            .completion_message,
        Some("my-message".to_owned())
    );
}

#[test]
async fn set_completion_status(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.completion_status, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    runner
        .set_completion_status(ctx, Some(ActionCompletionStatus::Success))
        .await
        .expect("unable to set completion status");
    assert_eq!(
        runner.completion_status,
        Some(ActionCompletionStatus::Success)
    );

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        ActionRunner::get_by_id(ctx, runner.id)
            .await
            .expect("unable to get action runner")
            .completion_status,
        Some(ActionCompletionStatus::Success)
    );
}

#[test]
async fn set_started_at(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert_eq!(runner.started_at, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    runner
        .set_started_at(ctx)
        .await
        .expect("unable to set completion status");
    assert!(runner.started_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(ActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner")
        .started_at
        .is_some());
}

#[test]
async fn set_finished_at(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert!(runner.finished_at.is_none());
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    runner
        .set_finished_at(ctx)
        .await
        .expect("unable to set completion status");
    assert!(runner.finished_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(ActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner")
        .finished_at
        .is_some());
}

#[test]
async fn stamp_started(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert!(runner.started_at.is_none());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    runner
        .stamp_started(ctx)
        .await
        .expect("unable to set stamp started");
    assert!(runner.started_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(ActionRunner::get_by_id(ctx, runner.id)
        .await
        .expect("unable to get action runner")
        .started_at
        .is_some());

    assert!(runner.stamp_started(ctx).await.is_err());
}

#[test]
async fn stamp_finished(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");
    assert!(runner.started_at.is_none());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(runner
        .stamp_finished(
            ctx,
            ActionCompletionStatus::Success,
            Some("message".to_owned()),
            Some(ActionRunResult {
                status: None,
                payload: None,
                message: None,
                logs: Vec::new(),
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
            Some(ActionRunResult {
                status: None,
                payload: None,
                message: None,
                logs: Vec::new(),
                last_synced: None,
            }),
        )
        .await
        .expect("unable to set stamp finished");
    assert!(runner.finished_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let runner = ActionRunner::get_by_id(ctx, runner.id)
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
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let runner = ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
        .await
        .expect("unable to create action runner");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let runners = ActionRunner::for_batch(ctx, batch.id)
        .await
        .expect("unable to get action runners for batch");
    assert_eq!(runners.len(), 1);
    assert_eq!(runners[0], runner);
}

#[test]
async fn run(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let mut runner =
        ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
            .await
            .expect("unable to create action runner");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(runner.run(ctx).await.expect("unable to run").is_some());
}
