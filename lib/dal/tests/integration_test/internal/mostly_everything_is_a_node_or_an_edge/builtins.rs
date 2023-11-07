use dal::DalContext;
use dal_test::test;

// TODO(nick): restore dal_test::helpers module to ensure the macro works.
#[test]
async fn builtins(ctx: &DalContext) {
    dbg!(
        ctx.tenancy(),
        ctx.visibility(),
        ctx.change_set_pointer()
            .expect("could not get change set pointer")
    );

    let mut snapshot = ctx
        .workspace_snapshot()
        .expect("could not get workspace snapshot")
        .lock()
        .await;
    snapshot.dot();
}
