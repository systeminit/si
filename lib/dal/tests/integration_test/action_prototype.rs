use dal::{ActionPrototype, ActionPrototypeContext, DalContext, WorkflowPrototypeId};
use pretty_assertions_sorted::assert_eq;

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext) {
    let name = "create";
    let context = ActionPrototypeContext::default();
    let prototype = ActionPrototype::new(ctx, WorkflowPrototypeId::NONE, name, context)
        .await
        .expect("unable to create action prototype");
    assert_eq!(prototype.name(), name);
    assert_eq!(prototype.workflow_prototype_id(), WorkflowPrototypeId::NONE);
}
