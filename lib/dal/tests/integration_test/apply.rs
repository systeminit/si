use dal::{
    ChangeSet,
    Component,
    DalContext,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        component,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

// Creates three components on three different change sets.
// Then, by spawning three tasks, applies them to head concurrently,
// once all the tasks are complete, we assert that all three components
// exist on head. This test is very slow!
#[test]
async fn concurrent_applies_to_head(ctx: &mut DalContext) -> Result<()> {
    // Create three components on three separate change sets
    let component_1_id = component::create(ctx, "swifty", "component_1").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSet::wait_for_dvu(ctx, false).await?;
    let change_set_1_id = ctx.change_set_id();

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let component_2_id = component::create(ctx, "swifty", "component_2").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSet::wait_for_dvu(ctx, false).await?;
    let change_set_2_id = ctx.change_set_id();

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let component_3_id = component::create(ctx, "swifty", "component_3").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSet::wait_for_dvu(ctx, false).await?;
    let change_set_3_id = ctx.change_set_id();

    // Spawn three concurrent tasks to apply each change set to head
    let handle_1 = {
        let mut ctx_clone = ctx.clone_with_change_set(change_set_1_id);
        tokio::spawn(async move {
            ChangeSetTestHelpers::apply_change_set_to_base(&mut ctx_clone)
                .await
                .expect("failed to apply change set 1");
        })
    };

    let handle_2 = {
        let mut ctx_clone = ctx.clone_with_change_set(change_set_2_id);
        tokio::spawn(async move {
            ChangeSetTestHelpers::apply_change_set_to_base(&mut ctx_clone)
                .await
                .expect("failed to apply change set 2");
        })
    };

    let handle_3 = {
        let mut ctx_clone = ctx.clone_with_change_set(change_set_3_id);
        tokio::spawn(async move {
            ChangeSetTestHelpers::apply_change_set_to_base(&mut ctx_clone)
                .await
                .expect("failed to apply change set 3");
        })
    };

    // Wait for all tasks to complete
    handle_1.await.expect("task 1 panicked");
    handle_2.await.expect("task 2 panicked");
    handle_3.await.expect("task 3 panicked");

    // Update context to HEAD and verify all three components exist
    let ctx = ctx.clone_with_head().await?;

    let component_1 = Component::get_by_id(&ctx, component_1_id).await?;
    assert_eq!("component_1", component_1.name(&ctx).await?);

    let component_2 = Component::get_by_id(&ctx, component_2_id).await?;
    assert_eq!("component_2", component_2.name(&ctx).await?);

    let component_3 = Component::get_by_id(&ctx, component_3_id).await?;
    assert_eq!("component_3", component_3.name(&ctx).await?);

    Ok(())
}
