use std::collections::HashMap;

use dal::{
    ComponentType,
    DalContext,
    Prop,
    SchemaVariant,
    prop::PropPath,
    property_editor::schema::PropertyEditorSchema,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        schema::variant,
    },
    test,
};
use itertools::Itertools as _;
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test]
async fn prop_path(ctx: &DalContext) -> Result<()> {
    let variant = SchemaVariant::default_for_schema_name(ctx, "starfield").await?;

    let name_path = PropPath::new(["root", "si", "name"]);
    let name_id = Prop::find_prop_id_by_path(ctx, variant.id(), &name_path).await?;
    let fetched_name_path = Prop::path_by_id(ctx, name_id).await?;

    assert_eq!(name_path, fetched_name_path);

    Ok(())
}

#[test]
async fn verify_prop_used_as_input_flag(ctx: &DalContext) -> Result<()> {
    let variant_id = SchemaVariant::default_id_for_schema_name(ctx, "pirate").await?;

    let container_props = [
        vec!["root"],
        vec!["root", "domain"],
        vec!["root", "domain", "parrot_names"],
        vec!["root", "domain", "treasure"],
    ];
    let item_props = [
        vec!["root", "domain", "parrot_names", "parrot_name"],
        vec!["root", "domain", "treasure", "location"],
    ];

    for container_prop_path in &container_props {
        let container_prop = Prop::get_by_id(
            ctx,
            Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(container_prop_path))
                .await?,
        )
        .await?;

        assert!(
            container_prop.can_be_used_as_prototype_arg,
            "{container_prop_path:?} should be marked as able to be used as a prototype argument"
        );
    }

    for item_prop_path in &item_props {
        let item_prop = Prop::get_by_id(
            ctx,
            Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(item_prop_path)).await?,
        )
        .await?;

        assert!(
            !item_prop.can_be_used_as_prototype_arg,
            "{item_prop_path:?} should be marked as NOT able to be used as a prototype argument"
        );
    }

    Ok(())
}

#[test]
async fn ordered_child_props(ctx: &DalContext) -> Result<()> {
    let variant_id = SchemaVariant::default_id_for_schema_name(ctx, "starfield").await?;

    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, variant_id).await?;
    let ordered_child_props = Prop::direct_child_props_ordered(ctx, root_prop_id).await?;
    let domain_prop = ordered_child_props
        .iter()
        .find(|p| p.name == "domain")
        .expect("could not find prop");
    let ordered_child_props = Prop::direct_child_props_ordered(ctx, domain_prop.id).await?;
    let ordered_child_prop_names = ordered_child_props
        .iter()
        .map(|p| p.name.to_owned())
        .collect_vec();

    let expected_child_prop_names = [
        "name",
        "hidden_prop",
        "freestar",
        "attributes",
        "possible_world_a",
        "possible_world_b",
        "universe",
    ];
    let expected_child_prop_names = expected_child_prop_names
        .iter()
        .map(|n| n.to_string())
        .collect_vec();

    assert_eq!(
        expected_child_prop_names, // expected
        ordered_child_prop_names   // actual
    );

    Ok(())
}

#[test]
async fn prop_documentation(ctx: &mut DalContext) -> Result<()> {
    let name = "Toto Wolff";
    let description = None;
    let link = None;
    let category = "Mercedes AMG Petronas";
    let color = "#00A19B";

    // Create an asset with a corresponding asset func. After that, commit.
    let schema_variant_id = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        name,
        description.clone(),
        link.clone(),
        category,
        color,
    )
    .await?
    .id();
    let asset_func = r##"function main() {
        const asset = new AssetBuilder();

        const alpha_source_prop = new PropBuilder()
            .setName("alpha_source_prop")
            .setKind("string")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();
        asset.addProp(alpha_source_prop);

        const alpha_destination_prop = new PropBuilder()
            .setName("alpha_destination_prop")
            .setKind("string")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();
        asset.addProp(alpha_destination_prop);

        const beta_source_prop = new PropBuilder()
            .setName("beta_source_prop")
            .setKind("string")
            .setDocumentation("sweet docs yo")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();
        asset.addProp(beta_source_prop);

        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName("beta_destination_output_socket")
            .setArity("one")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }"##;
    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant_id,
        name,
        name,
        category,
        description.clone(),
        link.clone(),
        color,
        ComponentType::Component,
        Some(asset_func),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Once it's all ready, regenerate and commit.
    let schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Assemble property editor schema and ensure Prop Documentation is there.
    let property_editor_schema =
        PropertyEditorSchema::assemble(ctx, schema_variant_id, false).await?;

    let prop = property_editor_schema
        .props
        .values()
        .find(|schema| schema.name == "beta_source_prop")
        .expect("could not find prop");

    assert_eq!(
        prop.documentation.as_ref().expect("has documentation"),
        "sweet docs yo"
    );

    // now let's add documentation for the other prop, regenerate, and make sure everything works

    let asset_func = r##"function main() {
        const asset = new AssetBuilder();

        const alpha_source_prop = new PropBuilder()
            .setName("alpha_source_prop")
            .setKind("string")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();
        asset.addProp(alpha_source_prop);

        const alpha_destination_prop = new PropBuilder()
            .setName("alpha_destination_prop")
            .setKind("string")
            .setDocumentation("more cool docs!")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();
        asset.addProp(alpha_destination_prop);

        const beta_source_prop = new PropBuilder()
            .setName("beta_source_prop")
            .setKind("string")
            .setDocumentation("sweet docs yo")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();
        asset.addProp(beta_source_prop);

        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName("beta_destination_output_socket")
            .setArity("one")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }"##;
    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant_id,
        name,
        name,
        category,
        description,
        link,
        color,
        ComponentType::Component,
        Some(asset_func),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Once it's all ready, regenerate and commit.
    let schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Assemble property editor schema and ensure both Prop Documentation is there.
    let property_editor_schema =
        PropertyEditorSchema::assemble(ctx, schema_variant_id, false).await?;

    let first_prop = property_editor_schema
        .props
        .values()
        .find(|schema| schema.name == "beta_source_prop")
        .expect("could not find prop");

    assert_eq!(
        first_prop
            .documentation
            .as_ref()
            .expect("has documentation"),
        "sweet docs yo"
    );

    let second_prop = property_editor_schema
        .props
        .values()
        .find(|schema| schema.name == "alpha_destination_prop")
        .expect("could not find prop");

    assert_eq!(
        second_prop
            .documentation
            .as_ref()
            .expect("has documentation"),
        "more cool docs!"
    );

    Ok(())
}

#[test]
async fn prop_suggestions(ctx: &DalContext) -> Result<()> {
    let foo = variant::create(
        ctx,
        "foo",
        r##"function main() { return new AssetBuilder()
                .addProp(new PropBuilder().setName("Id").setKind("string").build())
                .build();
        }"##,
    )
    .await?;

    assert_eq!(
        HashMap::new(),
        Prop::find_prop_by_path(ctx, foo, &PropPath::new(["root", "domain", "Id"]))
            .await?
            .ui_optionals
    );

    let bar = variant::create(
        ctx,
        "bar",
        r##"function main() { return new AssetBuilder()
            .addProp(new PropBuilder()
                .setName("Id")
                .setKind("string")
                .suggestAsSourceFor({ schema: "baz", prop: "BarId" })

                // Make sure you can have multiple suggestAsSourceFor
                .suggestAsSourceFor({ schema: "baz2", prop: "BarId2" })
                .build())
            .addProp(new PropBuilder()
                .setName("FooId")
                .setKind("string")
                .suggestSource({ schema: "foo", prop: "Id" })

                // Make sure you can have multiple suggestSource
                .suggestSource({ schema: "foo2", prop: "Id2" })
                // Make sure you can have suggestSource as well as suggestAsSourceFor on the same prop
                .suggestAsSourceFor({ schema: "baz", prop: "FooBarId" })
                .build())
            .build();
        }"##,
    )
    .await?;

    assert_eq!(
        HashMap::from([(
            "suggestAsSourceFor".to_string(),
            json!([
                { "schema": "baz", "prop": "BarId" },
                { "schema": "baz2", "prop": "BarId2" },
            ])
            .into()
        )]),
        Prop::find_prop_by_path(ctx, bar, &PropPath::new(["root", "domain", "Id"]))
            .await?
            .ui_optionals
    );

    assert_eq!(
        HashMap::from([
            (
                "suggestSources".to_string(),
                json!([
                    { "schema": "foo", "prop": "Id" },
                    { "schema": "foo2", "prop": "Id2" },
                ])
                .into()
            ),
            (
                "suggestAsSourceFor".to_string(),
                json!([
                    { "schema": "baz", "prop": "FooBarId" },
                ])
                .into()
            ),
        ]),
        Prop::find_prop_by_path(ctx, bar, &PropPath::new(["root", "domain", "FooId"]))
            .await?
            .ui_optionals
    );

    Ok(())
}
