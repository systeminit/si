use std::collections::HashSet;

use dal::{
    self,
    Component,
    ComponentId,
    DalContext,
    component::debug::ComponentDebugView,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        component,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn prevent_incomplete_subscriptions(ctx: &mut DalContext) -> Result<()> {
    // Create two components with an inter-component subscription.
    let erased_component_id = component::create(ctx, "Docker Image", "hammerfell").await?;
    let subscriber_component_id = component::create(ctx, "Docker Image", "highrock").await?;
    value::set(ctx, ("hammerfell", "/domain/image"), "neloth").await?;
    value::subscribe(
        ctx,
        ("highrock", "/domain/image"),
        ("hammerfell", "/domain/image"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        "neloth",                                              // expected
        value::get(ctx, ("highrock", "/domain/image")).await?, // actual
    );

    // Apply to HEAD and check the subscription.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    assert_eq!(
        "neloth",                                              // expected
        value::get(ctx, ("highrock", "/domain/image")).await?, // actual
    );

    // In a new change set, erase the component with the "source" value and commit.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let erase_change_set_id = ctx.change_set_id();
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, &[erased_component_id]).await?;
    let erased_component = Component::get_by_id(ctx, erased_component_id).await?;
    dal::component::delete::delete_component(ctx, &erased_component, true, &head_components)
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // In a second, new change set, override and then re-create the subscription. Commit in between each action.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    value::set(ctx, ("highrock", "/domain/image"), "overriden").await?;
    assert_eq!(
        "neloth",                                                // expected
        value::get(ctx, ("hammerfell", "/domain/image")).await?, // actual
    );
    assert_eq!(
        "overriden",                                           // expected
        value::get(ctx, ("highrock", "/domain/image")).await?, // actual
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    value::subscribe(
        ctx,
        ("highrock", "/domain/image"),
        ("hammerfell", "/domain/image"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        "neloth",                                              // expected
        value::get(ctx, ("highrock", "/domain/image")).await?, // actual
    );

    // Apply the second change set to HEAD and check the subscription.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    assert_eq!(
        "neloth",                                              // expected
        value::get(ctx, ("highrock", "/domain/image")).await?, // actual
    );

    // Switch back to the first change set and check that we can create a debug view. If we can,
    // then we know we have prevented an incomplete subscription from being made with our
    // correct transform logic.
    ChangeSetTestHelpers::switch_to_change_set(ctx, erase_change_set_id).await?;
    ComponentDebugView::new(ctx, subscriber_component_id).await?;

    Ok(())
}
