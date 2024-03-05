use pretty_assertions_sorted::assert_eq;

use dal::{ActionPrototype, ActionPrototypeContext, DalContext, FuncId, ActionKind};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let context = ActionPrototypeContext::default();
    let prototype = ActionPrototype::new(ctx, FuncId::NONE, ActionKind::Create, context)
        .await
        .expect("unable to create action prototype");
    assert_eq!(*prototype.kind(), ActionKind::Create);
    assert_eq!(prototype.func_id(), FuncId::NONE);
}
