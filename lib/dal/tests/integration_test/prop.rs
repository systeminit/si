use dal::{prop::PropPath, DalContext, Prop, Schema, SchemaVariant};
use dal_test::test;

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

    let pirate_default_variant_id = pirate_schema
        .get_default_schema_variant(ctx)
        .await
        .expect("should be able to get default")
        .expect("should have a default schema variant");

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
