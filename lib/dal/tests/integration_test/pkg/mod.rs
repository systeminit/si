use dal::pkg::export::PkgExporter;
use dal::pkg::import_pkg_from_pkg;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{DalContext, FuncBackendKind, FuncBackendResponseType};
use dal_test::test;
use si_pkg::{FuncSpec, FuncSpecData, PkgSpec, SchemaSpec, SchemaSpecData, SiPkg};

#[test]
async fn import_pkg_from_pkg_set_latest_default(ctx: &mut DalContext) {
    // Let's create a new asset
    let asset_name = "imanasset".to_string();
    let display_name = None;
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        display_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await
    .expect("Unable to create new asset");

    let schema = variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");

    assert!(default_schema_variant.is_some());
    assert_eq!(default_schema_variant, Some(variant.id()));

    // now lets create a pkg from the asset and import it
    let (variant_spec, variant_funcs) = PkgExporter::export_variant_standalone(ctx, &variant)
        .await
        .expect("should go to spec");

    let schema_spec = SchemaSpec::builder()
        .name(schema.name())
        .unique_id(schema.id())
        .variant(variant_spec)
        .data(
            SchemaSpecData::builder()
                .name(schema.name())
                .category(category.clone())
                .default_schema_variant(variant.id())
                .build()
                .expect("should build data"),
        )
        .build()
        .expect("should build spec");

    let func_spec = FuncSpec::builder()
        .name(asset_name.clone())
        .unique_id(schema.id())
        .data(
            FuncSpecData::builder()
                .name(asset_name.clone())
                .backend_kind(FuncBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncBackendResponseType::SchemaVariantDefinition)
                .handler("main")
                .code_plaintext("I am code")
                .build()
                .expect("should build data"),
        )
        .build()
        .expect("should make new func spec");

    let pkg_spec = PkgSpec::builder()
        .name(asset_name)
        .created_by("sally@systeminit.com")
        .funcs(variant_funcs)
        .func(func_spec)
        .schemas([schema_spec].to_vec())
        .version("0")
        .build()
        .expect("should build");

    let pkg = SiPkg::load_from_spec(pkg_spec).expect("should load from spec");

    // import and get add variants
    let (_, mut variants, _) = import_pkg_from_pkg(ctx, &pkg, None)
        .await
        .expect("should import");
    assert!(variants.len() == 1);

    let default_schema_variant = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");

    // the new default variant should be the one we just added
    assert!(default_schema_variant.is_some());
    assert_eq!(
        default_schema_variant,
        Some(variants.pop().expect("should pop"))
    );
}
