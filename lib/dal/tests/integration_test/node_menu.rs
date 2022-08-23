use crate::dal::test;
use dal::node_menu::get_node_menu_items;
use dal::test_harness::{create_component_for_schema, create_schema_variant};
use dal::DalContext;
use dal::StandardModel;
use dal::{node_menu::MenuFilter, Schema, SchematicKind};

#[test]
async fn get_node_menu(ctx: &DalContext<'_, '_>) {
    let application = create_component_for_schema(ctx, application_schema.id()).await;

    let items = get_node_menu_items(
        ctx,
        &MenuFilter::new(SchematicKind::Deployment, *application.id()),
    )
    .await
    .expect("cannot get items");

    let service_item = items.iter().find(|(_path, item)| item.name == "service");
    assert!(service_item.is_some(), "menu must include the service item");
}
