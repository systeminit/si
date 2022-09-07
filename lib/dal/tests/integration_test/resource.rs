use dal::DalContext;

use crate::dal::test;
use dal::test_harness::create_component_and_schema;
use dal::{Resource, StandardModel, SystemId, WorkflowPrototypeId};

#[test]
async fn new(ctx: &DalContext) {
    let component = create_component_and_schema(ctx).await;

    let _resource = Resource::new(
        ctx,
        *component.id(),
        SystemId::NONE,
        "key".to_owned(),
        serde_json::Value::Null,
        WorkflowPrototypeId::NONE,
    )
    .await
    .expect("cannot create resource for component/system");
}

#[test]
async fn list_by_component(ctx: &DalContext) {
    let component = create_component_and_schema(ctx).await;

    let resource = Resource::new(
        ctx,
        *component.id(),
        SystemId::NONE,
        "key".to_owned(),
        serde_json::Value::Null,
        WorkflowPrototypeId::NONE,
    )
    .await
    .expect("cannot create resource for component/system");
    let resources = Resource::list_by_component(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("unable to list resources");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0], resource);
}
