use dal::func::intrinsics::IntrinsicFunc;
use dal::DalContext;
use dal_test::test;
use strum::IntoEnumIterator;

// TODO(nick): restore dal_test::helpers module to ensure the macro works.
#[test]
async fn builtins(ctx: &DalContext) {
    let mut snapshot = ctx
        .workspace_snapshot()
        .expect("could not get workspace snapshot")
        .lock()
        .await;

    let mut funcs: Vec<String> = snapshot
        .list_funcs(ctx)
        .await
        .expect("list funcs should work")
        .iter()
        .map(|f| f.name.to_owned())
        .collect();

    let mut intrinsics: Vec<String> = IntrinsicFunc::iter()
        .map(|intrinsic| intrinsic.name().to_owned())
        .collect();

    funcs.sort();
    intrinsics.sort();

    assert_eq!(intrinsics, funcs);
}
