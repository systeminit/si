use dal::component::{DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::diagram::Diagram;
use dal::prop::{Prop, PropPath};
use dal::property_editor::values::PropertyEditorValues;
use dal::{AttributeValue, AttributeValueId};
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn update_and_insert_and_update(ctx: &mut DalContext) {
    let docker_image = Schema::list(ctx)
        .await
        .expect("list schemas")
        .iter()
        .find(|schema| schema.name() == "Docker Image")
        .expect("docker image does not exist")
        .to_owned();

    let variant = SchemaVariant::list_for_schema(ctx, docker_image.id())
        .await
        .expect("get schema variants")
        .pop()
        .expect("get default variant");

    let name = "a tulip in a cup";

    let component =
        Component::new(ctx, name, variant.id(), None).expect("able to create component");

    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let image_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant.id(),
        &PropPath::new(["root", "domain", "image"]),
    )
    .expect("able to find image prop");

    let exposed_ports_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant.id(),
        &PropPath::new(["root", "domain", "ExposedPorts"]),
    )
    .expect("able to find exposed ports prop");

    let exposed_ports_elem_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant.id(),
        &PropPath::new(["root", "domain", "ExposedPorts", "ExposedPort"]),
    )
    .expect("able to find exposed ports element prop");

    // Update image
    let image_av_id = property_values
        .values
        .iter()
        .find(|(_, v)| v.prop_id() == image_prop_id)
        .map(|(_, pvalue)| pvalue.attribute_value_id())
        .expect("can't find default attribute value for ExposedPorts");

    let image_value = serde_json::json!("fiona/apple");
    AttributeValue::update(ctx, image_av_id, Some(image_value.clone()))
        .await
        .expect("able to update image prop");

    let exposed_port_attribute_value_id = property_values
        .values
        .iter()
        .find(|(_, v)| v.prop_id() == exposed_ports_prop_id)
        .map(|(_, pvalue)| pvalue.attribute_value_id())
        .expect("can't find default attribute value for ExposedPorts");

    // Insert it unset first (to mimick frontend)
    let inserted_av_id = AttributeValue::insert(ctx, exposed_port_attribute_value_id, None, None)
        .await
        .expect("able to insert");

    // Before sending to the rebaser, confirm the value is there and it's the only one for the
    // ExposedPorts prop
    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let (fetched_image_value, image_av_id_again) = property_values
        .values
        .iter()
        .find(|(_, v)| v.prop_id() == image_prop_id)
        .map(|(_, v)| (v.value(), v.attribute_value_id()))
        .expect("able to get image av id from pvalues");

    assert_eq!(image_av_id, image_av_id_again);
    assert_eq!(image_value, fetched_image_value);

    let mut inserted_attribute_values: Vec<AttributeValueId> = property_values
        .values
        .iter()
        .filter_map(|(_, v)| {
            if v.prop_id() == exposed_ports_elem_prop_id {
                Some(v.attribute_value_id())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(1, inserted_attribute_values.len());
    let pvalues_inserted_attribute_value_id =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);

    // Rebase!
    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(matches!(conflicts, None));

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    // Confirm after rebase
    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let (fetched_image_value, image_av_id_again) = property_values
        .values
        .iter()
        .find(|(_, v)| v.prop_id() == image_prop_id)
        .map(|(_, v)| (v.value(), v.attribute_value_id()))
        .expect("able to get image av id from pvalues");

    assert_eq!(image_av_id, image_av_id_again);
    assert_eq!(image_value, fetched_image_value);

    let mut inserted_attribute_values: Vec<(serde_json::Value, AttributeValueId)> = property_values
        .values
        .iter()
        .filter_map(|(_, v)| {
            if v.prop_id() == exposed_ports_elem_prop_id {
                Some((v.value(), v.attribute_value_id()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, serde_json::Value::Null);

    let value = serde_json::json!("i ran out of white doves feathers");

    // Update the value we inserted
    AttributeValue::update(ctx, inserted_av_id, Some(value.clone()))
        .await
        .expect("able to update");

    // Confirm again before rebase
    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let mut inserted_attribute_values: Vec<(serde_json::Value, AttributeValueId)> = property_values
        .values
        .iter()
        .filter_map(|(_, v)| {
            if v.prop_id() == exposed_ports_elem_prop_id {
                Some((v.value(), v.attribute_value_id()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, value.clone());

    // Rebase again!
    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(matches!(conflicts, None));

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let mut inserted_attribute_values: Vec<(serde_json::Value, AttributeValueId)> = property_values
        .values
        .iter()
        .filter_map(|(_, v)| {
            if v.prop_id() == exposed_ports_elem_prop_id {
                Some((v.value(), v.attribute_value_id()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, value.clone());
}

#[test]
async fn create_and_determine_lineage(ctx: &DalContext) {
    // List all schemas in the workspace. Pick the first one alphabetically.
    let mut schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    let schema = schemas.pop().expect("schemas are empty");

    // Ensure we can get it by id.
    let found_schema = Schema::get_by_id(ctx, schema.id())
        .await
        .expect("could not get schema by id");
    assert_eq!(
        schema.id(),       // expected
        found_schema.id()  // actual
    );

    // Pick a schema variant.
    let mut schema_variants = SchemaVariant::list_for_schema(ctx, found_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let schema_variant = schema_variants.pop().expect("schemas are empty");
    let schema_variant_id = schema_variant.id();

    // Create a component and set geometry.
    let name = "fsu not top four";
    let component =
        Component::new(ctx, name, schema_variant_id, None).expect("could not create component");
    let component = component
        .set_geometry(
            ctx,
            "1",
            "-1",
            Some(DEFAULT_COMPONENT_WIDTH),
            Some(DEFAULT_COMPONENT_HEIGHT),
        )
        .await
        .expect("could not set geometry");

    // Determine the schema variant from the component. Ensure it is the same as before.
    let post_creation_schema_variant = Component::schema_variant(ctx, component.id())
        .await
        .expect("could not get schema variant for component");
    assert_eq!(
        schema_variant_id,                 // expected
        post_creation_schema_variant.id()  // actual
    );

    // Determine the schema from the schema variant. Ensure it is the same as before.
    let post_creation_schema = SchemaVariant::schema(ctx, post_creation_schema_variant.id())
        .await
        .expect("could not get schema for schema variant");
    assert_eq!(
        schema.id(),               // expected
        post_creation_schema.id()  // actual
    );

    // Assemble the diagram just to make sure it works.
    let _diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
}
