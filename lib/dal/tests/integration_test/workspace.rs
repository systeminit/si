use dal::change_set::view::OpenChangeSetsView;
use dal::diagram::Diagram;
use dal::{DalContext, Workspace};
use dal_test::helpers::{
    create_component_for_schema_name, ChangeSetTestHelpers, PropEditorTestView,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn export_import_loop(ctx: &mut DalContext) {
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("assemble view");

    let initial_change_set_count = view.change_sets.len();

    let change_set_name = "exported".to_string();
    ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, &change_set_name)
        .await
        .expect("fork change set");

    // Create Component With Value
    let original_pirate_name = "Cap'n Crunch ";
    create_component_for_schema_name(ctx, "pirate", original_pirate_name).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot to visibility");

    let workspace_pk = ctx.tenancy().workspace_pk().expect("find workspace pk");
    let mut workspace = Workspace::get_by_pk(ctx, &workspace_pk)
        .await
        .expect("execute find workspace")
        .expect("find workspace");

    // Export changeset
    let workspace_export = workspace
        .generate_export_data(ctx, "0.0")
        .await
        .expect("export workspace");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot to visibility");

    ChangeSetTestHelpers::abandon_change_set(ctx)
        .await
        .expect("abandon change set");

    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("assemble view");

    // Check that we no longer have the abandoned changeset
    assert_eq!(
        initial_change_set_count, // expected
        view.change_sets.len()    // actual
    );

    workspace
        .import(ctx, workspace_export)
        .await
        .expect("import changeset");

    // Import
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that we have imported the changeset
    assert_eq!(
        initial_change_set_count + 1, // expected
        view.change_sets.len()        // actual
    );

    let imported_change_set_id = view
        .change_sets
        .iter()
        .find(|cs| cs.name == change_set_name)
        .expect("find changeset")
        .id;

    ctx.update_visibility_and_snapshot_to_visibility(imported_change_set_id)
        .await
        .expect("update context to use imported data");

    // Check that the component exists with the correct value
    let diagram = Diagram::assemble(ctx).await.expect("load diagram");

    // Check that the component exists
    assert_eq!(
        1,                        // expected
        diagram.components.len()  // actual
    );

    let component = diagram.components.first().expect("get component");

    let name_path = &["root", "domain", "name"];
    assert_eq!(
        original_pirate_name, // expected
        PropEditorTestView::for_component_id(ctx, component.id) //actual
            .await
            .get_value(name_path)
            .get("value")
            .expect("get value for domain/name")
    );
}
