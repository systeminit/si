use dal::DalContext;

use crate::dal::test;
use dal::node_menu::{get_node_menu_items, GenerateMenuItem};
use dal::schema::SchemaKind;
use dal::test_harness::{create_schema, create_schema_ui_menu};
use dal::{schema::UiMenu, Component, MenuFilter, Schema, SchematicKind, StandardModel};
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let schema_ui_menu = UiMenu::new(ctx, &SchematicKind::Component)
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(schema_ui_menu.name(), None);
    assert_eq!(schema_ui_menu.category(), None);
    assert_eq!(schema_ui_menu.schematic_kind(), &SchematicKind::Component);
}

#[test]
async fn set_schema(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let schema_ui_menu = create_schema_ui_menu(ctx).await;

    schema_ui_menu
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = schema_ui_menu
        .schema(ctx)
        .await
        .expect("cannot get schema")
        .expect("should have a schema");
    assert_eq!(schema, attached_schema);

    schema_ui_menu
        .unset_schema(ctx)
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = schema_ui_menu.schema(ctx).await.expect("cannot get schema");
    assert_eq!(attached_schema, None);
}

#[test]
async fn root_schematics(ctx: &DalContext<'_, '_>) {
    let root_schema = create_schema(ctx, &SchemaKind::Concrete).await;

    let schema_ui_menu = create_schema_ui_menu(ctx).await;

    schema_ui_menu
        .add_root_schematic(ctx, root_schema.id())
        .await
        .expect("cannot add root schematic");

    let root_schematics = schema_ui_menu
        .root_schematics(ctx)
        .await
        .expect("cannot list root schematics");
    assert_eq!(root_schematics, vec![root_schema.clone()]);

    schema_ui_menu
        .remove_root_schematic(ctx, root_schema.id())
        .await
        .expect("cannot add root schematic");
    let no_root_schematics = schema_ui_menu
        .root_schematics(ctx)
        .await
        .expect("cannot list root schematics");
    assert_eq!(no_root_schematics, vec![]);
}

#[test]
async fn three_layers(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Concrete).await;

    let application_name = "application";
    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name)
        .await
        .expect("unable to find application");
    let application_schema = application_schema_results
        .first()
        .expect("unable to find application schema");

    let (application_component, _node) =
        Component::new_application_with_node(ctx, "application".to_owned())
            .await
            .expect("unable to create application");

    let mut root_ui_menu = create_schema_ui_menu(ctx).await;
    root_ui_menu
        .set_name(ctx, Some("root-test".to_owned()))
        .await
        .expect("unable to set name");
    root_ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await
        .expect("unable to set root schematic");

    let mut parent_ui_menu = create_schema_ui_menu(ctx).await;
    parent_ui_menu
        .set_name(ctx, Some("parent-test".to_owned()))
        .await
        .expect("unable to set name");
    parent_ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await
        .expect("unable to set root schematic");
    parent_ui_menu
        .set_category(ctx, Some("root-test".to_owned()))
        .await
        .expect("unable to set category");

    // root-test->parent-test->schema.name()
    let mut schema_ui_menu = create_schema_ui_menu(ctx).await;
    schema_ui_menu
        .set_name(ctx, Some(schema.name().to_string()))
        .await
        .expect("unable to set name");
    schema_ui_menu
        .set_category(ctx, Some("root-test.parent-test".to_owned()))
        .await
        .expect("unable to set category");
    schema_ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await
        .expect("unable to set root schematic");
    schema_ui_menu
        .set_schema(ctx, schema.id())
        .await
        .expect("unable to set schema");

    let items = get_node_menu_items(
        &ctx,
        &MenuFilter {
            schematic_kind: SchematicKind::Component,
            root_component_id: *application_component.id(),
        },
    )
    .await
    .expect("unable to get_node_menu_items");
    let response = {
        let gmi = GenerateMenuItem::new();
        gmi.create_menu_json(items)
            .expect("unable to generate menu json")
    };

    assert_eq_sorted!(
        response
            .as_array()
            .expect("response was not an array")
            .iter()
            .find(|obj| obj
                .get("name")
                .expect("unable to get name")
                .as_str()
                .expect("name wasnt a string")
                == "root".to_owned())
            .expect("root not found"),
        &serde_json::json!({
            "kind": "category",
            "name": "root-test",
            "items": [{
                "kind": "category",
                "name": "parent-test",
                "items": [{
                    "kind": "item",
                    "name": schema.name(),
                    "items": []
                }]
            }],
        })
    );
}
