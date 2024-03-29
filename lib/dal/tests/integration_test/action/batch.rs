use dal::{
    ActionBatch, ActionCompletionStatus, ActionPrototype, ActionRunner, Component, DalContext,
};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn runners(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(batch
        .runners(ctx)
        .await
        .expect("unable to list runners")
        .is_empty());

    ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
        .await
        .expect("unable to create action runner");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        batch
            .runners(ctx)
            .await
            .expect("unable to list runners")
            .len(),
        1
    );
}

#[test]
async fn get_by_id(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        ActionBatch::get_by_id(ctx, batch.id)
            .await
            .expect("unable to get action batch"),
        batch
    );
}

#[test]
async fn set_completion_status(ctx: &mut DalContext) {
    let mut batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    assert_eq!(batch.completion_status, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    batch
        .set_completion_status(ctx, Some(ActionCompletionStatus::Success))
        .await
        .expect("unable to set completion status");
    assert_eq!(
        batch.completion_status,
        Some(ActionCompletionStatus::Success)
    );

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert_eq!(
        ActionBatch::get_by_id(ctx, batch.id)
            .await
            .expect("unable to get action batch")
            .completion_status,
        Some(ActionCompletionStatus::Success)
    );
}

#[test]
async fn set_started_at(ctx: &mut DalContext) {
    let mut batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    assert_eq!(batch.started_at, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    batch
        .set_started_at(ctx)
        .await
        .expect("unable to set completion status");
    assert!(batch.started_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(ActionBatch::get_by_id(ctx, batch.id)
        .await
        .expect("unable to get action batch")
        .started_at
        .is_some());
}

#[test]
async fn set_finished_at(ctx: &mut DalContext) {
    dbg!("yo");
    let mut batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    dbg!("made batch");
    assert_eq!(batch.finished_at, None);

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    dbg!("blocking commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");
    dbg!("update snap");

    batch
        .set_finished_at(ctx)
        .await
        .expect("unable to set completion status");
    assert!(batch.finished_at.is_some());
    dbg!("set finished at");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(ActionBatch::get_by_id(ctx, batch.id)
        .await
        .expect("unable to get action batch")
        .finished_at
        .is_some());
}

#[test]
async fn stamp_started(ctx: &mut DalContext) {
    let mut batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    assert!(batch.started_at.is_none());

    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    ActionRunner::new(ctx, batch.id, component.id(), "swifty".to_owned(), proto.id)
        .await
        .expect("unable to create action runner");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    batch
        .stamp_started(ctx)
        .await
        .expect("unable to set stamp started");
    assert!(batch.started_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(ActionBatch::get_by_id(ctx, batch.id)
        .await
        .expect("unable to get action batch")
        .started_at
        .is_some());

    assert!(batch.stamp_started(ctx).await.is_err());
}

#[test]
async fn stamp_finished(ctx: &mut DalContext) {
    let mut batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");
    assert!(batch.started_at.is_none());

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
    runner
        .set_completion_status(ctx, Some(ActionCompletionStatus::Success))
        .await
        .expect("unable to set completion status");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    assert!(batch.stamp_finished(ctx).await.is_err());
    batch
        .stamp_started(ctx)
        .await
        .expect("unable to set stamp started");
    batch
        .stamp_finished(ctx)
        .await
        .expect("unable to set stamp finished");
    assert!(batch.finished_at.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let batch = ActionBatch::get_by_id(ctx, batch.id)
        .await
        .expect("unable to get action batch");
    assert!(batch.finished_at.is_some());
    assert_eq!(
        batch.completion_status,
        Some(ActionCompletionStatus::Success)
    );
}

#[test]
async fn list(ctx: &mut DalContext) {
    let batch = ActionBatch::new(ctx, "batch", "paulo was here")
        .await
        .expect("unable to create action batch");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let batches = ActionBatch::list(ctx)
        .await
        .expect("unable to get action batch");
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], batch);
}
