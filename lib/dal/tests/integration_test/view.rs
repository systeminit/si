use dal::{
    Component,
    DalContext,
    Ulid,
    WorkspaceSnapshotError,
    diagram::{
        Diagram,
        DiagramError,
        geometry::Geometry,
        view::View,
    },
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
};
use dal_test::{
    expected::{
        self,
        ExpectView,
        generate_fake_name,
    },
    helpers::{
        create_component_for_default_schema_name,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_frontend_types::RawGeometry;

#[test(skip_rebaser, skip_pinga)]
async fn create_view_and_component(ctx: &mut DalContext) {
    let alternative_view = ExpectView::create(ctx).await;

    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        alternative_view.id(),
    )
    .await
    .expect("could not create component");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(default_diagram.components.len(), 0);

    let alternative_diagram = Diagram::assemble(ctx, Some(alternative_view.id()))
        .await
        .expect("assemble default diagram");

    assert_eq!(alternative_diagram.components.len(), 1);

    let diagram_component = alternative_diagram
        .components
        .first()
        .expect("component from alternative diagram");

    assert_eq!(
        diagram_component.id.as_raw_id(),
        component.id().into_inner()
    )
}

#[test(skip_rebaser, skip_pinga)]
async fn deleting_component_deletes_geometries(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await
            .expect("could not create component");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(default_diagram.components.len(), 1);

    let diagram_component = default_diagram
        .components
        .first()
        .expect("component from alternative diagram");

    assert_eq!(
        diagram_component.id.as_raw_id(),
        component.id().into_inner()
    );

    // Add component to another view
    let another_view = ExpectView::create(ctx).await;

    Component::add_to_view(
        ctx,
        component.id(),
        another_view.id(),
        RawGeometry {
            x: 0,
            y: 0,
            width: None,
            height: None,
        },
    )
    .await
    .expect("add component to view");

    let maybe_marked_component = component.delete(ctx).await.expect("component deleted");
    // Ensure the component got deleted instead
    // of marked for deletion
    assert_eq!(maybe_marked_component, None);

    // Ensure we get an empty diagram with no failures, which dangling geometries would cause
    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(default_diagram.components.len(), 0);

    let another_diagram = Diagram::assemble(ctx, Some(another_view.id()))
        .await
        .expect("assemble another diagram");

    assert_eq!(another_diagram.components.len(), 0);
}

#[test(skip_rebaser, skip_pinga)]
async fn remove_view_with_no_components(ctx: &mut DalContext) {
    let new_view = ExpectView::create(ctx).await;

    create_component_for_default_schema_name_in_default_view(ctx, "swifty", generate_fake_name())
        .await
        .expect("could not create component");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(1, default_diagram.components.len());

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list views").len()
    );

    let alternative_diagram = Diagram::assemble(ctx, Some(new_view.id()))
        .await
        .expect("assemble default diagram");

    assert_eq!(0, alternative_diagram.components.len());

    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(1, default_diagram.components.len());

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list views").len(),
        "view is removed",
    );
}

#[test(skip_rebaser, skip_pinga, skip_veritech)]
async fn remove_view_with_no_exclusive_components(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "swifty",
        generate_fake_name(),
    )
    .await
    .expect("could not create component");

    let new_view = ExpectView::create(ctx).await;

    Geometry::new_for_component(ctx, component.id(), new_view.id())
        .await
        .expect("Unable to create Geometry for Component in new View");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(1, default_diagram.components.len());

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list views").len()
    );

    let alternative_diagram = Diagram::assemble(ctx, Some(new_view.id()))
        .await
        .expect("assemble default diagram");

    assert_eq!(1, alternative_diagram.components.len());

    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(1, default_diagram.components.len());

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list views").len()
    );
}

#[test(skip_rebaser, skip_pinga, skip_veritech)]
async fn remove_view_with_exclusive_components(ctx: &mut DalContext) {
    let new_view = ExpectView::create(ctx).await;

    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        new_view.id(),
    )
    .await
    .expect("could not create component");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(0, default_diagram.components.len());

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list views").len()
    );

    let alternative_diagram = Diagram::assemble(ctx, Some(new_view.id()))
        .await
        .expect("assemble default diagram");

    assert_eq!(1, alternative_diagram.components.len());

    let result = View::remove(ctx, new_view.id()).await;
    let orphans = match result {
        Err(DiagramError::WorkspaceSnapshot(err)) => match *err {
            WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                WorkspaceSnapshotGraphError::ViewRemovalWouldOrphanItems(orphans),
            )
            | WorkspaceSnapshotError::ViewRemovalWouldOrphanItems(orphans) => orphans,
            _ => panic!("View removal did not error appropriately: {err:?}"),
        },
        _ => panic!("View removal did not error appropriately: {result:?}"),
    };
    assert_eq!(vec![Ulid::from(component.id())], orphans,);
}

#[test(skip_rebaser, skip_pinga, skip_veritech)]
async fn remove_view_with_removal_of_exclusive_components(ctx: &mut DalContext) {
    let new_view = ExpectView::create(ctx).await;

    let component = create_component_for_default_schema_name(
        ctx,
        "swifty",
        generate_fake_name(),
        new_view.id(),
    )
    .await
    .expect("could not create component");

    let default_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("assemble default diagram");

    assert_eq!(0, default_diagram.components.len());

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list views").len()
    );

    let alternative_diagram = Diagram::assemble(ctx, Some(new_view.id()))
        .await
        .expect("assemble default diagram");

    assert_eq!(1, alternative_diagram.components.len());

    Component::remove(ctx, component.id())
        .await
        .expect("Unable to remove Component");
    View::remove(ctx, new_view.id())
        .await
        .expect("Unable to remove View");

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list views").len()
    );
}

#[test]
async fn remove_view_that_previously_contained_another_view_that_has_been_removed(
    ctx: &mut DalContext,
) {
    let containing_view = ExpectView::create(ctx).await;
    let contained_view = ExpectView::create(ctx).await;
    assert_eq!(
        0,
        Geometry::list_by_view_id(ctx, containing_view.id())
            .await
            .expect("Unable to list Geometry for containing View")
            .len()
    );
    View::add_to_another_view(
        ctx,
        contained_view.id(),
        containing_view.id(),
        RawGeometry::default(),
    )
    .await
    .expect("Unable to add contained view to container view");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    assert_eq!(
        3,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );

    assert_eq!(
        1,
        Geometry::list_by_view_id(ctx, containing_view.id())
            .await
            .expect("Unable to list Geometry for containing View")
            .len()
    );

    View::remove(ctx, contained_view.id())
        .await
        .expect("Unable to remove contained View");

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );

    assert_eq!(
        0,
        Geometry::list_by_view_id(ctx, containing_view.id())
            .await
            .expect("Unable to list Geometry for containing View")
            .len()
    );

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    assert_eq!(
        2,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );

    assert_eq!(
        0,
        Geometry::list_by_view_id(ctx, containing_view.id())
            .await
            .expect("Unable to list Geometry for containing View")
            .len()
    );

    View::remove(ctx, containing_view.id())
        .await
        .expect("Unable to remove containing View");

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    assert_eq!(
        1,
        View::list(ctx).await.expect("Unable to list Views").len(),
    );
}
