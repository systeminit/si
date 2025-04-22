use dal::{
    Component,
    DalContext,
    diagram::Diagram,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};

#[test]
async fn components_removed_from_snapshot_have_virtual_diagram_entries(ctx: &mut DalContext) {
    let component_to_remove = create_component_for_default_schema_name_in_default_view(
        ctx,
        "starfield",
        "Removed in sub-change set",
    )
    .await
    .expect("Unable to create component.");
    let _component_still_in_change_set =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", "Still here")
            .await
            .expect("Unable to create component.");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to commit");
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("Unable to merge setup to HEAD");

    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("Unable to create forked change set");

    let removed_component = Component::get_by_id(ctx, component_to_remove.id())
        .await
        .expect("Unable to get component to remove");
    removed_component
        .delete(ctx)
        .await
        .expect("Unable to remove component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to commit");

    assert!(
        Component::get_by_id(ctx, component_to_remove.id())
            .await
            .is_err()
    );

    let summary_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("Unable to assemble summary diagram");

    let removed_component_summary = summary_diagram
        .components
        .iter()
        .find(|comp| comp.id == component_to_remove.id())
        .expect("Removed Component not found in summary diagram");
    assert!(removed_component_summary.from_base_change_set);

    assert_eq!(
        si_frontend_types::ChangeStatus::Deleted,
        removed_component_summary.change_status
    );
}
