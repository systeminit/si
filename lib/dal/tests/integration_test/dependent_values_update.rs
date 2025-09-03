use dal::{
    Component,
    DalContext,
    component::resource::ResourceData,
    workspace_snapshot::DependentValueRoot,
};
use dal_test::{
    Result,
    helpers::{
        attribute::value,
        change_set,
        component,
    },
    test,
};
use serde_json::json;
use veritech_client::ResourceStatus;

// TODO implement this! It worked for connections but does not work for subscriptions
#[ignore]
#[test]
async fn marked_for_deletion_to_normal_is_blocked(ctx: &mut DalContext) -> Result<()> {
    // Create a marked_for_deletion and normal component, and connect them.
    component::create(ctx, "Docker Image", "ghost").await?;
    component::create(ctx, "Docker Image", "normal").await?;
    value::set(ctx, ("ghost", "/domain/image"), "a").await?;
    value::subscribe(ctx, ("normal", "/domain/image"), ("ghost", "/domain/image")).await?;
    change_set::commit(ctx).await?;
    // Downstream value is passed down the subscription.
    assert_eq!("a", value::get(ctx, ("normal", "/domain/image")).await?);

    // Mark the ghost component for deletion (give it a resource so it isn't immediately deleted)
    let ghost = Component::get_by_id(ctx, component::id(ctx, "ghost").await?).await?;
    ghost
        .set_resource(
            ctx,
            ResourceData::new(
                ResourceStatus::Ok,
                Some(serde_json::json![{"resource": "something"}]),
            ),
        )
        .await?;
    ghost
        .delete(ctx)
        .await?
        .expect("component got fully deleted instead of just ghosted");
    change_set::commit(ctx).await?;

    // Downstream value is null!
    assert_eq!(
        json!(null),
        value::get(ctx, ("normal", "/domain/image")).await?
    );

    // Modify ghosted component's value and verify downstream value is still null.
    value::set(ctx, ("ghost", "/domain/image"), "b").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!(null),
        value::get(ctx, ("normal", "/domain/image")).await?
    );

    // Undelete ghosted component and verify downstream value is updated
    let ghost = Component::get_by_id(ctx, component::id(ctx, "ghost").await?).await?;
    ghost.set_to_delete(ctx, false).await?;
    change_set::commit(ctx).await?;
    assert_eq!("b", value::get(ctx, ("normal", "/domain/image")).await?);
    Ok(())
}

#[test]
async fn normal_to_marked_for_deletion_flows(ctx: &mut DalContext) -> Result<()> {
    // Create a marked_for_deletion and normal component
    component::create(ctx, "Docker Image", "ghost").await?;
    component::create(ctx, "Docker Image", "normal").await?;
    value::set(ctx, ("normal", "/domain/image"), "a").await?;
    value::subscribe(ctx, ("ghost", "/domain/image"), ("normal", "/domain/image")).await?;
    change_set::commit(ctx).await?;
    // Downstream value is passed down the subscription.
    assert_eq!("a", value::get(ctx, ("ghost", "/domain/image")).await?);

    // Mark the ghost component for deletion (give it a resource so it isn't immediately deleted)
    let ghost = Component::get_by_id(ctx, component::id(ctx, "ghost").await?).await?;
    ghost
        .set_resource(
            ctx,
            ResourceData::new(
                ResourceStatus::Ok,
                Some(serde_json::json![{"resource": "something"}]),
            ),
        )
        .await?;
    ghost
        .delete(ctx)
        .await?
        .expect("component got fully deleted instead of just ghosted");
    change_set::commit(ctx).await?;

    // Downstream value is still set!
    assert_eq!("a", value::get(ctx, ("ghost", "/domain/image")).await?);

    // Modify upstream component's value and verify downstream value is updated
    value::set(ctx, ("normal", "/domain/image"), "b").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("b"),
        value::get(ctx, ("ghost", "/domain/image")).await?
    );

    Ok(())
}

/// Until we have a better system for signalling that a DVU has run and
/// finished, we can't actually verify that it executed these per-component. But
/// we can ensure that with a concurrency limit: (1) the job finishes and (2) it
/// produces the correct data
#[test]
async fn component_concurrency_limit(ctx: &mut DalContext) -> Result<()> {
    // Give us a massive component concurrency level
    let mut workspace = ctx.get_workspace().await?;
    workspace
        .set_component_concurrency_limit(ctx, Some(10000))
        .await?;
    ctx.commit_no_rebase().await?;

    // create 1 etoile, and 16 morningstars
    component::create(ctx, "Docker Image", "source").await?;

    let mut downstreams = vec![];
    for i in 0..16 {
        let id = component::create(ctx, "Docker Image", (i + 1).to_string()).await?;
        value::subscribe(ctx, (id, "/domain/image"), ("source", "/domain/image")).await?;
        downstreams.push(id);
    }

    assert!(
        DependentValueRoot::roots_exist(ctx).await?,
        "should have dvu roots to be processed"
    );
    change_set::commit(ctx).await?;

    assert!(
        !DependentValueRoot::roots_exist(ctx).await?,
        "all dvu roots should be processed and removed"
    );

    let mut workspace = ctx.get_workspace().await?;
    workspace
        .set_component_concurrency_limit(ctx, Some(2))
        .await?;
    ctx.commit_no_rebase().await?;

    value::set(ctx, ("source", "/domain/image"), "phosphorus").await?;

    change_set::commit(ctx).await?;

    assert!(
        !DependentValueRoot::roots_exist(ctx).await?,
        "all roots should be processed and off the graph"
    );

    for id in downstreams {
        assert_eq!("phosphorus", value::get(ctx, (id, "/domain/image")).await?);
    }

    Ok(())
}
