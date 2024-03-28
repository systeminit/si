use dal::{prop::PropPath, AttributeValue, Component, DalContext, Prop, Schema, SchemaVariant};
use dal_test::test;
use dal_test::test_harness::{commit_and_update_snapshot, create_component_for_schema_name};

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
async fn prop_validation(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "pirate", "Robinson Crusoe").await;

    let sv_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("get sv for component");

    let prop_path = PropPath::new(["root", "domain", "working_eyes"]);
    let prop_id = Prop::find_prop_id_by_path(ctx, sv_id, &prop_path)
        .await
        .expect("get name prop id");

    let av_id = component
        .attribute_values_for_prop(ctx, &["root", "domain", "working_eyes"])
        .await
        .expect("find value ids for the prop")
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(3)))
        .await
        .expect("override domain/parrot_names attribute value");

    commit_and_update_snapshot(ctx).await;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(5)))
        .await
        .expect("override domain/parrot_names attribute value");

    commit_and_update_snapshot(ctx).await;
}
