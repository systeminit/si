use dal::change_set_pointer::ChangeSetPointer;
use dal::DalContext;
use dal_test::test;
use ulid::Ulid;

#[test]
async fn vector_clock_id(ctx: &mut DalContext) {
    let change_set = ChangeSetPointer::new(ctx, "main")
        .await
        .expect("could not create change set");
    let vector_clock_id_as_ulid: Ulid = change_set.vector_clock_id().into();
    assert_eq!(change_set.id, vector_clock_id_as_ulid.into());
}
