use dal::DalContext;

use crate::dal::test;
use dal::test_harness::create_component_and_schema;
use dal::{Resource, StandardModel, System};

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let component = create_component_and_schema(ctx).await;
    let system = System::new(ctx, "production system")
        .await
        .expect("cannot create system");

    let _resource = Resource::new(ctx, component.id(), system.id())
        .await
        .expect("cannot create resource for component/system");
}

#[test]
async fn get_by_component_and_system_id(ctx: &DalContext<'_, '_, '_>) {
    let mastodon_component = create_component_and_schema(ctx).await;
    let blue_oyster_component = create_component_and_schema(ctx).await;

    let production_system = System::new(ctx, "production system")
        .await
        .expect("cannot create system");
    let staging_system = System::new(ctx, "staging system")
        .await
        .expect("cannot create staging system");
    let original_resource = Resource::new(ctx, mastodon_component.id(), production_system.id())
        .await
        .expect("cannot create resource for component/system");

    // None of the following should be found by `Resource::get_by_component_id_and_system_id`.
    let _different_component_in_same_system =
        Resource::new(ctx, blue_oyster_component.id(), production_system.id())
            .await
            .expect("cannot create resource for different component in same system");
    let _same_component_in_different_system =
        Resource::new(ctx, mastodon_component.id(), staging_system.id())
            .await
            .expect("cannot create resource for same component in different system");
    let _different_component_in_different_system =
        Resource::new(ctx, blue_oyster_component.id(), staging_system.id())
            .await
            .expect("cannot create resource for different component in different system");

    let found_resource = Resource::get_by_component_id_and_system_id(
        ctx,
        mastodon_component.id(),
        production_system.id(),
    )
    .await
    .expect("cannot retrieve resource for component/system");

    let found_resource = found_resource.expect("unable to get resource from component and system");
    assert_eq!(
        original_resource, found_resource,
        "Resource::get_by_component_id_and_system_id needs to find the same resource we created"
    )
}
