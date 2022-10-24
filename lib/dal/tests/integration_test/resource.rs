use dal::{DalContext, Resource, StandardModel, SystemId};
use dal_test::{test, test_harness::create_component_and_schema};

#[test]
async fn new(ctx: &DalContext) {
    let component = create_component_and_schema(ctx).await;

    let resource = Resource::new(
        ctx,
        serde_json::Value::Null,
        *component.id(),
        SystemId::NONE,
    )
    .await
    .expect("cannot create resource for component/system");
    let found_resource =
        Resource::get_by_component_and_system(ctx, *component.id(), SystemId::NONE)
            .await
            .expect("unable to get resource")
            .expect("resource not found");
    assert_eq!(found_resource, resource);
}
