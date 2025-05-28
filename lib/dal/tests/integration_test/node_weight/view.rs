use dal::{
    Component, DalContext,
    diagram::{geometry::Geometry, view::View},
    workspace_snapshot::node_weight::{NodeWeight, category_node_weight::CategoryNodeKind},
};
use dal_test::{
    Result,
    expected::{self, ExpectView, generate_fake_name},
    helpers::create_component_for_default_schema_name,
    prelude::OptionExt,
    test,
};
use petgraph::Direction::Outgoing;
use pretty_assertions_sorted::assert_eq;
use si_frontend_types::RawGeometry;
use si_split_graph::SplitGraphNodeId;

#[test]
async fn correct_transforms_remove_view_all_geometries_removed(ctx: &mut DalContext) {
    let default_view_id = View::get_id_for_default(ctx)
        .await
        .expect("Unable to get default ViewId");
    let new_view = ExpectView::create(ctx).await;
    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        new_view.id(),
    )
    .await
    .expect("Unable to create Component in View");

    expected::apply_change_set_to_base(ctx).await;
    expected::fork_from_head_change_set(ctx).await;

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list Views").len()
    );

    assert_eq!(
        1,
        Geometry::list_by_view_id(ctx, new_view.id())
            .await
            .expect("Unable to list Geometries for View")
            .len(),
    );

    Component::remove(ctx, component.id())
        .await
        .expect("Unable to remove Component");

    let views = View::list(ctx).await.expect("Unable to list Views");
    assert_eq!(2, views.len());

    assert_eq!(
        0,
        Geometry::list_by_view_id(ctx, new_view.id())
            .await
            .expect("Unable to list Geometries for View")
            .len(),
    );

    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove view");

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len()
    );

    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        0,
        Geometry::list_by_view_id(ctx, default_view_id)
            .await
            .expect("Unable to list Geometries for default View")
            .len()
    );
    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len()
    );
}

#[test]
async fn correct_transforms_remove_view_already_removed_view(ctx: &mut DalContext) {
    let new_view = ExpectView::create(ctx).await;
    expected::apply_change_set_to_base(ctx).await;

    let change_set_one = expected::fork_from_head_change_set(ctx).await;
    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View in ChangeSet one");

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len()
    );

    let _change_set_two = expected::fork_from_head_change_set(ctx).await;
    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View from ChangeSet two");

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len()
    );

    expected::apply_change_set_to_base(ctx).await;
    ctx.update_visibility_and_snapshot_to_visibility(change_set_one.id)
        .await
        .expect("Unable to switch to ChangeSet one");
    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len()
    );
}

#[test]
async fn correct_transforms_remove_view_not_all_geometries_removed(ctx: &mut DalContext) {
    let new_view = ExpectView::create(ctx).await;
    let views = View::list(ctx).await.expect("Unable to list Views");
    assert_eq!(2, views.len());
    expected::apply_change_set_to_base(ctx).await;

    let view_removal_change_set = expected::fork_from_head_change_set(ctx).await;
    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View in ChangeSet");
    assert_eq!(
        1,
        View::list(ctx)
            .await
            .expect("Unable to list Views in view removal ChangeSet")
            .len(),
    );
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    expected::fork_from_head_change_set(ctx).await;

    create_component_for_default_schema_name(ctx, "swifty", generate_fake_name(), new_view.id())
        .await
        .expect("Unable to create component");

    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(
        2,
        View::list(ctx)
            .await
            .expect("Unable to list Views in base ChangeSet")
            .len(),
    );
    assert_eq!(
        1,
        Geometry::list_by_view_id(ctx, new_view.id())
            .await
            .expect("Unable to list Geometries in new View")
            .len(),
    );

    ctx.update_visibility_and_snapshot_to_visibility(view_removal_change_set.id)
        .await
        .expect("Unable to switch to view removal ChangeSet");

    assert_eq!(
        1,
        View::list(ctx)
            .await
            .expect("Unable to list Views in view removal ChangeSet")
            .len(),
    );

    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        2,
        View::list(ctx)
            .await
            .expect("Unable to list Views in base ChangeSet")
            .len(),
    );
}

async fn find_diagram_object_category(ctx: &DalContext) -> Option<SplitGraphNodeId> {
    let snap = ctx.workspace_snapshot().expect("tesco");
    let root_id = snap.root().await.expect("must have a root");
    for (_, _, target) in snap
        .edges_directed(root_id, Outgoing)
        .await
        .expect("root node does not exist?")
    {
        if let Some(NodeWeight::Category(cat)) = snap.get_node_weight_opt(target).await {
            if cat.kind() == CategoryNodeKind::DiagramObject {
                return Some(target);
            }
        }
    }

    None
}

#[test]
async fn correct_transforms_remove_view_no_other_views(ctx: &mut DalContext) {
    let new_view = ExpectView::create(ctx).await;
    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list Views").len()
    );
    assert_eq!(
        0,
        Geometry::list_by_view_id(ctx, new_view.id())
            .await
            .expect("Unable to list Geometries in new View")
            .len(),
    );

    let view_removal_change_set = expected::fork_from_head_change_set(ctx).await;
    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View in ChangeSet");
    find_diagram_object_category(ctx)
        .await
        .expect("expected diagram object category");

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    expected::fork_from_head_change_set(ctx).await;

    // change set with component, view not removed
    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        new_view.id(),
    )
    .await
    .expect("Unable to create Component in new View");

    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        1,
        Geometry::list_by_view_id(ctx, new_view.id())
            .await
            .expect("Unable to list Geometries in new View")
            .len(),
    );

    Component::get_by_id(ctx, component.id())
        .await
        .expect("component was not there!");

    ctx.update_visibility_and_snapshot_to_visibility(view_removal_change_set.id)
        .await
        .expect("Unable to switch to View removal ChangeSet");

    Component::get_by_id(ctx, component.id())
        .await
        .expect("component was not there in removal change set!");

    assert_eq!(
        1,
        View::list(ctx)
            .await
            .expect("Unable to list Views in base ChangeSet")
            .len(),
    );
    expected::apply_change_set_to_base(ctx).await;

    Component::get_by_id(ctx, component.id())
        .await
        .expect("component was not there in base change set!");
    assert_eq!(
        2,
        View::list(ctx)
            .await
            .expect("Unable to list Views in base ChangeSet")
            .len(),
    );
    assert_eq!(
        1,
        Geometry::list_by_view_id(ctx, new_view.id())
            .await
            .expect("Unable to list Geometries in new View")
            .len(),
    );

    let default_view_id = View::get_id_for_default(ctx)
        .await
        .expect("default view id");

    Geometry::new_for_view(ctx, new_view.id(), default_view_id)
        .await
        .expect("create geometry");
}

#[test]
async fn correct_transforms_remove_view_with_component_in_another_view(ctx: &mut DalContext) {
    let default_view_id = View::get_id_for_default(ctx)
        .await
        .expect("get default view");
    let new_view = ExpectView::create(ctx).await;
    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        default_view_id,
    )
    .await
    .expect("Unable to create Component in default View");
    Component::add_to_view(ctx, component.id(), new_view.id(), RawGeometry::default())
        .await
        .expect("Unable to add Component to new View");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    assert_eq!(
        2,
        Geometry::list_ids_by_component(ctx, component.id())
            .await
            .expect("Unable to get Geometries for Component")
            .len()
    );

    expected::fork_from_head_change_set(ctx).await;

    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove new View");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    assert_eq!(
        1,
        Geometry::list_ids_by_component(ctx, component.id())
            .await
            .expect("Unable to get Geometries for Component")
            .len()
    );

    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    assert_eq!(
        1,
        Geometry::list_ids_by_component(ctx, component.id())
            .await
            .expect("Unable to get Geometries for Component")
            .len()
    );
}

#[test]
async fn correct_transforms_remove_view_components_moved_to_another_view(
    ctx: &mut DalContext,
) -> Result<()> {
    let default_view_id = View::get_id_for_default(ctx).await?;
    let new_view = ExpectView::create(ctx).await;
    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        new_view.id(),
    )
    .await
    .expect("Unable to create Component in new View");
    expected::apply_change_set_to_base(ctx).await;
    expected::fork_from_head_change_set(ctx).await;

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    assert_eq!(
        1,
        Geometry::list_ids_by_component(ctx, component.id())
            .await?
            .len(),
    );

    // Move the Component to a different View, and remove the View it was originally in.
    let new_view_geometry_id = *Geometry::list_ids_by_component(ctx, component.id())
        .await?
        .first()
        .ok_or_eyre("Unable to retrieve first element of a single element Vec")?;
    Component::add_to_view(ctx, component.id(), default_view_id, RawGeometry::default()).await?;
    Geometry::remove(ctx, new_view_geometry_id).await?;
    View::remove(ctx, new_view.id()).await?;

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    assert_eq!(
        1,
        Geometry::list_ids_by_component(ctx, component.id())
            .await?
            .len(),
    );
    expected::apply_change_set_to_base(ctx).await;

    // Applying the set of transforms should see that even though the View being removed
    // was the only one the Component was in, it is also being added to a different View
    // in the same set of updates to apply, so it is not being orphaned.
    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    assert_eq!(
        1,
        Geometry::list_ids_by_component(ctx, component.id())
            .await?
            .len(),
    );

    Ok(())
}

/// Creates a view named "test view" and applies it to head. Then, creates two change sets.
///  In each change set, use Geometry::new_for_view to create a new geometry object in the
/// "test view" that represents the default view. Confirm no duplicate geometries for the
/// diagram objects are created.
///
/// This is really a correction on the "geometry" node weight, not the "view" node weight.
#[test]
async fn correct_transforms_no_duplicate_diagram_object_geometries(ctx: &mut DalContext) {
    let mut geometry_ids = vec![];

    let default_view_id = View::get_id_for_default(ctx)
        .await
        .expect("Unable to get default ViewId");

    // Create a view named "test view"
    let test_view = ExpectView::create_with_name(ctx, "test view").await;
    expected::apply_change_set_to_base(ctx).await;

    // Create first change set
    let first_change_set = expected::fork_from_head_change_set(ctx).await;

    // Create a new geometry in "test view" that represents the default view
    let geo_id = Geometry::new_for_view(ctx, default_view_id, test_view.id())
        .await
        .expect("Unable to create geometry in first change set")
        .id();
    geometry_ids.push(geo_id);

    let geometries = Geometry::list_by_view_id(ctx, test_view.id())
        .await
        .expect("Unable to list geometries for test view");

    assert_eq!(
        1,
        geometries.len(),
        "Expected only one geometry in test view"
    );

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Create second change set
    expected::fork_from_head_change_set(ctx).await;

    // Create another geometry in "test view" that represents the default view
    let geo_id = Geometry::new_for_view(ctx, default_view_id, test_view.id())
        .await
        .expect("Unable to create geometry in second change set")
        .id();
    geometry_ids.push(geo_id);

    // Apply second change set to head
    expected::apply_change_set_to_base(ctx).await;

    // Verify no duplicate geometires
    let geometries = Geometry::list_by_view_id(ctx, test_view.id())
        .await
        .expect("Unable to list geometries for test view");

    assert_eq!(
        1,
        geometries.len(),
        "Expected only one geometry in test view"
    );

    ctx.update_visibility_and_snapshot_to_visibility(first_change_set.id)
        .await
        .expect("Unable to update visibility and snapshot");

    let geometries = Geometry::list_by_view_id(ctx, test_view.id())
        .await
        .expect("Unable to list geometries for test view");

    assert_eq!(
        1,
        geometries.len(),
        "Expected only one geometry in test view"
    );

    ctx.workspace_snapshot()
        .expect("hello")
        .cleanup_and_merkle_tree_hash()
        .await
        .expect("hello");

    expected::apply_change_set_to_base(ctx).await;

    let geometries = Geometry::list_by_view_id(ctx, test_view.id())
        .await
        .expect("Unable to list geometries for test view");

    assert_eq!(
        1,
        geometries.len(),
        "Expected only one geometry in test view"
    );
}
