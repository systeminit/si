use dal::{
    ComponentType, DalContext, Prop, Schema, SchemaVariant, prop::PropPath,
    property_editor::schema::PropertyEditorSchema,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{helpers::ChangeSetTestHelpers, test};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn prop_path(ctx: &DalContext) {
    let starfield_schema = Schema::list(ctx)
        .await
        .expect("list schemas")
        .iter()
        .find(|schema| schema.name() == "starfield")
        .expect("starfield does not exist")
        .to_owned();

    let variant = SchemaVariant::list_for_schema(ctx, starfield_schema.id())
        .await
        .expect("get schema variants")
        .pop()
        .expect("get default variant");

    let name_path = PropPath::new(["root", "si", "name"]);
    let name_id = Prop::find_prop_id_by_path(ctx, variant.id(), &name_path)
        .await
        .expect("get name prop id");
    let fetched_name_path = Prop::path_by_id(ctx, name_id)
        .await
        .expect("get prop path by id");

    assert_eq!(name_path, fetched_name_path);
}

#[test]
async fn verify_prop_used_as_input_flag(ctx: &DalContext) {
    let pirate_schema = Schema::list(ctx)
        .await
        .expect("list schemas")
        .iter()
        .find(|schema| schema.name() == "pirate")
        .expect("pirate does not exist")
        .to_owned();

    let pirate_default_variant_id = Schema::default_variant_id(ctx, pirate_schema.id())
        .await
        .expect("should be able to get default");

    let _pirate = SchemaVariant::get_by_id(ctx, pirate_default_variant_id)
        .await
        .expect("should be able to get pirate sv");

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
            Prop::find_prop_id_by_path(
                ctx,
                pirate_default_variant_id,
                &PropPath::new(container_prop_path),
            )
            .await
            .expect("should have the container prop"),
        )
        .await
        .expect("id should resolve to a prop");

        assert!(
            container_prop.can_be_used_as_prototype_arg,
            "{:?} should be marked as able to be used as a prototype argument",
            container_prop_path
        );
    }

    for item_prop_path in &item_props {
        let item_prop = Prop::get_by_id(
            ctx,
            Prop::find_prop_id_by_path(
                ctx,
                pirate_default_variant_id,
                &PropPath::new(item_prop_path),
            )
            .await
            .expect("should have the item prop"),
        )
        .await
        .expect("id should resolve to a prop");

        assert!(
            !item_prop.can_be_used_as_prototype_arg,
            "{:?} should be marked as NOT able to be used as a prototype argument",
            item_prop_path
        );
    }
}

#[test]
async fn ordered_child_props(ctx: &DalContext) {
    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id)
        .await
        .expect("could not get root prop id");
    let ordered_child_props = Prop::direct_child_props_ordered(ctx, root_prop_id)
        .await
        .expect("could not get direct child props ordered");
    let domain_prop = ordered_child_props
        .iter()
        .find(|p| p.name == "domain")
        .expect("could not find prop");
    let ordered_child_props = Prop::direct_child_props_ordered(ctx, domain_prop.id)
        .await
        .expect("could not get direct child props ordered");
    let ordered_child_prop_names: Vec<String> = ordered_child_props
        .iter()
        .map(|p| p.name.to_owned())
        .collect();

    let expected_child_prop_names = [
        "name",
        "hidden_prop",
        "freestar",
        "attributes",
        "possible_world_a",
        "possible_world_b",
        "universe",
    ];
    let expected_child_prop_names: Vec<String> = expected_child_prop_names
        .iter()
        .map(|n| n.to_string())
        .collect();

    assert_eq!(
        expected_child_prop_names, // expected
        ordered_child_prop_names   // actual
    );
}

#[test]
async fn prop_documentation(ctx: &mut DalContext) {
    let name = "Toto Wolff";
    let description = None;
    let link = None;
    let category = "Mercedes AMG Petronas";
    let color = "#00A19B";

    // Create an asset with a corresponding asset func. After that, commit.
    let schema_variant_id = {
        let schema_variant = VariantAuthoringClient::create_schema_and_variant(
            ctx,
            name,
            description.clone(),
            link.clone(),
            category,
            color,
        )
        .await
        .expect("unable to create schema and variant");
        schema_variant.id()
    };
    let asset_func = "function main() {
        const asset = new AssetBuilder();

        const alpha_source_prop = new PropBuilder()
            .setName(\"alpha_source_prop\")
            .setKind(\"string\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(alpha_source_prop);

        const alpha_destination_prop = new PropBuilder()
            .setName(\"alpha_destination_prop\")
            .setKind(\"string\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(alpha_destination_prop);

        const beta_source_prop = new PropBuilder()
            .setName(\"beta_source_prop\")
            .setKind(\"string\")
            .setDocumentation(\"sweet docs yo\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(beta_source_prop);

        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName(\"beta_destination_output_socket\")
            .setArity(\"one\")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }";
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
    .await
    .expect("could not save content");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Once it's all ready, regenerate and commit.
    let schema_variant_id = VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id)
        .await
        .expect("could not regenerate variant");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Assemble property editor schema and ensure Prop Documentation is there.
    let property_editor_schema = PropertyEditorSchema::assemble(ctx, schema_variant_id, false)
        .await
        .expect("could not assemble property editor schema");

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

    let asset_func = "function main() {
        const asset = new AssetBuilder();

        const alpha_source_prop = new PropBuilder()
            .setName(\"alpha_source_prop\")
            .setKind(\"string\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(alpha_source_prop);

        const alpha_destination_prop = new PropBuilder()
            .setName(\"alpha_destination_prop\")
            .setKind(\"string\")
            .setDocumentation(\"more cool docs!\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(alpha_destination_prop);

        const beta_source_prop = new PropBuilder()
            .setName(\"beta_source_prop\")
            .setKind(\"string\")
            .setDocumentation(\"sweet docs yo\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(beta_source_prop);

        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName(\"beta_destination_output_socket\")
            .setArity(\"one\")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }";
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
    .await
    .expect("could not save content");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Once it's all ready, regenerate and commit.
    let schema_variant_id = VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id)
        .await
        .expect("could not regenerate variant");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Assemble property editor schema and ensure both Prop Documentation is there.
    let property_editor_schema = PropertyEditorSchema::assemble(ctx, schema_variant_id, false)
        .await
        .expect("could not assemble property editor schema");

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
}
