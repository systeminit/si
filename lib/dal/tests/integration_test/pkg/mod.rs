use dal::{
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentId,
    DalContext,
    FuncBackendKind,
    FuncBackendResponseType,
    Prop,
    PropId,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    pkg::{
        ImportOptions,
        export::PkgExporter,
        import_pkg_from_pkg,
    },
    prop::PropPath,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    expected::ExpectSchemaVariant,
    helpers::create_component_for_schema_variant_on_default_view,
    test,
};
use si_pkg::{
    FuncSpec,
    FuncSpecData,
    PkgSpec,
    PropSpec,
    SchemaSpec,
    SchemaSpecData,
    SiPkg,
};

#[test]
async fn import_pkg_from_pkg_set_latest_default(ctx: &mut DalContext) -> Result<()> {
    // Let's create a new asset
    let asset_name = "imanasset".to_string();
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await?;

    let schema = variant.schema(ctx).await?;

    let default_schema_variant = Schema::default_variant_id(ctx, schema.id()).await?;

    assert_eq!(default_schema_variant, variant.id());

    // now lets create a pkg from the asset and import it
    let (variant_spec, variant_funcs) =
        PkgExporter::export_variant_standalone(ctx, &variant, schema.name(), None).await?;

    let schema_spec = SchemaSpec::builder()
        .name(schema.name())
        .unique_id(schema.id())
        .variant(variant_spec)
        .data(
            SchemaSpecData::builder()
                .name(schema.name())
                .category(category.clone())
                .default_schema_variant(variant.id())
                .build()?,
        )
        .build()?;

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
                .build()?,
        )
        .build()?;

    let pkg_spec = PkgSpec::builder()
        .name(asset_name)
        .created_by("sally@systeminit.com")
        .funcs(variant_funcs)
        .func(func_spec)
        .schemas([schema_spec].to_vec())
        .version("0")
        .build()?;

    let pkg = SiPkg::load_from_spec(pkg_spec).expect("should load from spec");

    // import and get add variants
    let (_, mut variants, _) = import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema.id().into()),
            ..Default::default()
        }),
    )
    .await?;
    assert_eq!(variants.len(), 1);

    let default_schema_variant = Schema::default_variant_id(ctx, schema.id()).await?;

    // the new default variant should be the one we just added
    assert_eq!(default_schema_variant, variants.pop().expect("should pop"));

    Ok(())
}

#[test]
async fn prop_order_preserved(ctx: &mut DalContext) -> Result<()> {
    let expected_props = vec![
        "foo",
        "bar",
        "bar.john",
        "bar.jacob",
        "bar.jingleheimer",
        "bar.smith",
        "baz",
    ];

    let schema_id = {
        // Create a variant with a particular prop order
        let variant_id = ExpectSchemaVariant::create_named(
            ctx,
            "testme",
            r#"function main() {
                return {
                    props: [
                        { name: "foo", kind: "string" },
                        { name: "bar", kind: "object", children: [
                                { name: "john", kind: "string" },
                                { name: "jacob", kind: "string" },
                                { name: "jingleheimer", kind: "string" },
                                { name: "smith", kind: "string" },
                        ] },
                        { name: "baz", kind: "string" },
                    ]
                };
            }"#,
        )
        .await
        .id();
        assert_eq!(variant_prop_names(ctx, variant_id).await?, expected_props,);

        // Create a component and check that order is preserved
        let component_id = create_component_for_schema_variant_on_default_view(ctx, variant_id)
            .await?
            .id();
        assert_eq!(
            component_prop_names(ctx, component_id).await?,
            expected_props,
        );
        SchemaVariant::schema_id(ctx, variant_id).await?
    };

    // Export variant -> PkgSpec
    let exported_spec = PkgExporter::new_for_module_contribution(
        "testme",
        "test_version",
        "me@me.com",
        schema_id,
        false,
    )
    .export_as_spec(ctx)
    .await?;
    assert_eq!(spec_prop_names(&exported_spec), expected_props);

    // PkgSpec -> SiPkg
    let exported_pkg = SiPkg::load_from_spec(exported_spec)?;
    assert_eq!(
        // check that order is preserved by converting back to spec
        spec_prop_names(&exported_pkg.to_spec().await?),
        expected_props
    );

    // Round trip SiPkg -> bytes -> SiPkg
    let exported_bytes = exported_pkg.write_to_bytes()?;
    let pkg = SiPkg::load_from_bytes(&exported_bytes)?;
    // check that order is preserved
    assert_eq!(spec_prop_names(&pkg.to_spec().await?), expected_props);

    // Check that the SiPkg -> variant has the same prop order
    {
        let pkg_variant_id = {
            let (_, pkg_variant_ids, _) = import_pkg_from_pkg(
                ctx,
                &pkg,
                Some(ImportOptions {
                    schema_id: Some(schema_id.into()),
                    ..Default::default()
                }),
            )
            .await?;
            assert!(pkg_variant_ids.len() == 1);
            pkg_variant_ids[0]
        };
        assert_eq!(
            variant_prop_names(ctx, pkg_variant_id).await?,
            expected_props
        );

        // Create a component and check that order is preserved
        let pkg_component_id =
            create_component_for_schema_variant_on_default_view(ctx, pkg_variant_id)
                .await?
                .id();
        assert_eq!(
            component_prop_names(ctx, pkg_component_id).await?,
            expected_props
        );
    }

    // Check that the SiPkg -> variant -> component has the same prop order
    Ok(())
}

async fn variant_prop_names(
    ctx: &mut DalContext,
    variant_id: SchemaVariantId,
) -> Result<Vec<String>> {
    let domain_path = PropPath::new(["root", "domain"]);
    let domain_id = Prop::find_prop_id_by_path(ctx, variant_id, &domain_path).await?;
    child_prop_names(ctx, domain_id, None).await
}

async fn child_prop_names(
    ctx: &mut DalContext,
    parent_prop_id: PropId,
    prefix: Option<&str>,
) -> Result<Vec<String>> {
    let mut result = vec![];
    for Prop { name, id, .. } in Prop::direct_child_props_ordered(ctx, parent_prop_id).await? {
        let name = match prefix {
            Some(prefix) => format!("{prefix}.{name}"),
            None => name.to_owned(),
        };
        result.push(name.clone());
        result.extend(Box::pin(child_prop_names(ctx, id, Some(&name))).await?);
    }
    Ok(result)
}

async fn component_prop_names(
    ctx: &mut DalContext,
    component_id: ComponentId,
) -> Result<Vec<String>> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let domain_av_id = component.domain_prop_attribute_value(ctx).await?;
    child_av_names(ctx, domain_av_id, None).await
}

async fn child_av_names(
    ctx: &mut DalContext,
    parent_av_id: AttributeValueId,
    prefix: Option<&str>,
) -> Result<Vec<String>> {
    let mut result = vec![];
    for child_av_id in AttributeValue::get_child_av_ids_in_order(ctx, parent_av_id).await? {
        let name = AttributeValue::prop_name(ctx, child_av_id).await?;
        let name = match prefix {
            Some(prefix) => format!("{prefix}.{name}"),
            None => name.to_owned(),
        };
        result.push(name.clone());
        result.extend(Box::pin(child_av_names(ctx, child_av_id, Some(&name))).await?);
    }
    Ok(result)
}

fn spec_prop_names(spec: &PkgSpec) -> Vec<String> {
    spec_prop_child_names(&spec.schemas[0].variants[0].domain, None)
}

fn spec_prop_child_names(parent_prop: &PropSpec, prefix: Option<&str>) -> Vec<String> {
    let mut result = vec![];
    for child_prop in parent_prop.direct_children() {
        let name = child_prop.name();
        let name = match prefix {
            Some(prefix) => format!("{prefix}.{name}"),
            None => name.to_owned(),
        };
        result.push(name.clone());
        result.extend(spec_prop_child_names(child_prop, Some(&name)));
    }
    result
}
