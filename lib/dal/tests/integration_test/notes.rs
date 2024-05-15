use dal::{note::Note, DalContext};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn note(ctx: &DalContext) {
    let note_contents = "This is a demo note".to_string();
    let x_coord = "100".to_string();
    let y_coord = "100".to_string();
    let created_by = "sally@systeminit.com".to_string();

    let note = Note::new(
        ctx,
        x_coord.clone(),
        y_coord.clone(),
        note_contents.clone(),
        created_by,
    )
    .await
    .expect("unable to create note");

    assert_eq!(note_contents, note.note());
    assert_eq!(x_coord, note.x());
    assert_eq!(y_coord, note.y());

    let notes_in_changeset = Note::list(ctx).await.expect("Unable to list notes");

    assert_eq!(1, notes_in_changeset.len());
}
