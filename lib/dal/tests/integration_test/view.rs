use dal::diagram::Diagram;
use dal::{Component, DalContext};
use dal_test::expected::{generate_fake_name, ExpectView};
use dal_test::helpers::create_component_for_default_schema_name;
use dal_test::helpers::create_component_for_default_schema_name_in_default_view;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use si_frontend_types::RawGeometry;

#[test]
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

#[test]
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
        &ctx,
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
