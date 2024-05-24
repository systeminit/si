use dal::change_set::view::OpenChangeSetsView;
use dal::DalContext;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use std::collections::HashSet;

#[test]
async fn open_change_sets(ctx: &mut DalContext) {
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of open change sets exist.
    assert_eq!(
        2,                      // expected
        view.change_sets.len()  // actual
    );

    // Check that we collected "head" properly.
    let head_change_set_id = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("could not get the default change set id for the workspace");
    assert_eq!(
        head_change_set_id,      // expected
        view.head_change_set_id  // actual
    );

    // Ensure that the current change set is not "head".
    let current_change_set_id = ctx.change_set_id();
    assert_ne!(current_change_set_id, head_change_set_id);

    // Ensure that the views contain the change sets that we expect.
    let change_set_ids = HashSet::from_iter(view.change_sets.iter().map(|c| c.id));
    assert_eq!(
        HashSet::from([current_change_set_id, head_change_set_id]), // expected
        change_set_ids,                                             // actual
    );

    // Apply the change set and perform a blocking commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Assemble the view again and ensure only "head" exists.
    let mut view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of open change sets exist. There should only be one.
    let head_change_set_view = view.change_sets.pop().expect("change sets are empty");
    assert!(view.change_sets.is_empty());
    assert_eq!(
        view.head_change_set_id, // expected
        head_change_set_view.id  // actual
    );
    assert_eq!(
        head_change_set_id,      // expected
        head_change_set_view.id  // actual
    );

    // Create a new change set and perform a commit without rebasing.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork change set");

    // List views again.
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of open change sets exist... again.
    assert_eq!(
        2,                      // expected
        view.change_sets.len()  // actual
    );

    // Check that we collected "head" properly... again.
    let head_change_set_id_again = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("could not get the default change set id for the workspace");
    assert_eq!(
        head_change_set_id,       // expected
        head_change_set_id_again  // actual
    );
    assert_eq!(
        head_change_set_id,      // expected
        view.head_change_set_id  // actual
    );

    // Ensure that the current change set is not "head"... again.
    let current_change_set_id_again = ctx.change_set_id();
    assert_ne!(head_change_set_id, current_change_set_id_again);
    assert_ne!(current_change_set_id, current_change_set_id_again);

    // Ensure that the views contain the change sets that we expect... again.
    let change_set_ids_again = HashSet::from_iter(view.change_sets.iter().map(|c| c.id));
    assert_eq!(
        HashSet::from([current_change_set_id_again, head_change_set_id_again]), // expected
        change_set_ids_again,                                                   // actual
    );
}

#[test]
async fn abandon_change_set_and_check_open_change_sets(ctx: &mut DalContext) {
    let change_set_name = "for abandonment".to_string();
    ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, &change_set_name)
        .await
        .expect("could not fork change set");

    // List open changesets.
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of change sets exist....
    assert_eq!(
        3,                      // expected
        view.change_sets.len()  // actual
    );

    ChangeSetTestHelpers::abandon_change_set(ctx)
        .await
        .expect("could not abandon change set");

    // relist the open changesets.
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that we no longer have the abandoned changeset
    assert_eq!(
        2,                      // expected
        view.change_sets.len()  // actual
    );

    let change_set_names = Vec::from_iter(view.change_sets.iter().map(|c| c.name.clone()));
    assert!(!change_set_names.contains(&change_set_name))
}
