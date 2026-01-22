use chrono::Utc;
use dal::{
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentId,
    DalContext,
    Func,
    FuncBackendKind,
    FuncBackendResponseType,
    Prop,
    PropId,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    func::intrinsics::IntrinsicFunc,
    module::Module,
    pkg::{
        ImportOptions,
        Thing,
        ThingMap,
        UpdateExisting,
        export::PkgExporter,
        import_func,
        import_funcs_for_module_update,
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

#[test(enable_veritech)]
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

    // Generate a unique ID for this func (not reusing schema ID)
    let func_unique_id = ulid::Ulid::new();

    let func_spec = FuncSpec::builder()
        .name(asset_name.clone())
        .unique_id(func_unique_id)
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

#[test(enable_veritech)]
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

/// Test that `import_funcs_for_module_update` correctly handles intrinsic functions.
/// Intrinsic functions should be looked up by name rather than recreated.
#[test(enable_veritech)]
async fn import_funcs_for_module_update_handles_intrinsics(ctx: &mut DalContext) -> Result<()> {
    // Create a schema with variant
    let asset_name = "upgrade_test_asset".to_string();
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        None,
        None,
        "Integration Tests".to_string(),
        "#00b0b0".to_string(),
    )
    .await?;

    let schema = variant.schema(ctx).await?;

    // Export the variant to a pkg spec
    let mut exporter = PkgExporter::new_for_module_contribution(
        asset_name.clone(),
        "1.0.0",
        "test@test.com",
        schema.id(),
        false,
    );
    let exported_spec = exporter.export_as_spec(ctx).await?;
    let pkg = SiPkg::load_from_spec(exported_spec)?;

    // Get the funcs from the package
    let funcs = pkg.funcs()?;

    // Collect intrinsic func specs with their unique_ids
    let intrinsic_funcs: Vec<_> = funcs
        .iter()
        .filter(|f| IntrinsicFunc::maybe_from_str(f.name()).is_some())
        .map(|f| (f.unique_id().to_string(), f.name().to_string()))
        .collect();

    assert!(
        !intrinsic_funcs.is_empty(),
        "Expected exported pkg to contain intrinsic functions"
    );

    // Use the actual function we're testing
    let thing_map = import_funcs_for_module_update(ctx, funcs).await?;

    // Verify all intrinsic functions are in the thing_map
    for (unique_id, func_name) in &intrinsic_funcs {
        let thing = thing_map
            .get(unique_id)
            .unwrap_or_else(|| panic!("Expected func {func_name} to be in thing_map"));

        let Thing::Func(func) = thing else {
            panic!("Expected Thing::Func, got something else for {func_name}");
        };

        assert_eq!(&func.name, func_name, "Expected func name to match");
    }

    Ok(())
}

/// Test that `import_func` with UpdateExisting mode updates an existing func's code.
#[test(enable_veritech)]
async fn import_func_with_update_existing_updates_code(ctx: &mut DalContext) -> Result<()> {
    // Create a schema with variant (this creates a SchemaVariantDefinition func)
    let asset_name = "update_existing_test".to_string();
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        None,
        None,
        "Integration Tests".to_string(),
        "#00b0b0".to_string(),
    )
    .await?;

    let schema = variant.schema(ctx).await?;

    // Get the func from the variant
    let func_id = variant
        .asset_func_id()
        .expect("Expected variant to have an asset func");
    let func = Func::get_by_id(ctx, func_id).await?;
    let func_code_before = func.code_plaintext()?.expect("Expected func to have code");

    // Create a module and associate the func with it
    let module = Module::new(
        ctx,
        &asset_name,
        "test_hash_123",
        "1.0.0",
        "Test module",
        "test@test.com",
        Utc::now(),
        Some(schema.id().into()),
    )
    .await?;
    module.create_association(ctx, func.id.into()).await?;

    // Export the variant to a pkg spec
    let mut exporter = PkgExporter::new_for_module_contribution(
        asset_name.clone(),
        "1.0.0",
        "test@test.com",
        schema.id(),
        false,
    );
    let exported_spec = exporter.export_as_spec(ctx).await?;

    // Find the schema variant definition func in the exported spec
    let original_func_spec = exported_spec
        .funcs
        .iter()
        .find(|f| f.name == func.name)
        .expect("Expected to find asset func in exported spec");

    let original_data = original_func_spec
        .data
        .as_ref()
        .expect("Expected func to have data");

    // Build a new func spec with modified code
    let new_code = "function main() { return { props: [] }; }";
    let modified_func_spec = FuncSpec::builder()
        .name(&func.name)
        .unique_id(original_func_spec.unique_id.clone())
        .data(
            FuncSpecData::builder()
                .name(&func.name)
                .handler(&original_data.handler)
                .code_plaintext(new_code)
                .backend_kind(original_data.backend_kind)
                .response_type(original_data.response_type)
                .build()?,
        )
        .build()?;

    // Load the modified spec into an SiPkg
    let modified_pkg_spec = PkgSpec::builder()
        .name(&asset_name)
        .created_by("test@test.com")
        .func(modified_func_spec)
        .version("1.0.1")
        .build()?;
    let modified_pkg = SiPkg::load_from_spec(modified_pkg_spec)?;

    // Get the func from the package
    let pkg_funcs = modified_pkg.funcs()?;
    let pkg_func = pkg_funcs
        .iter()
        .find(|f| f.name() == func.name)
        .expect("Expected to find func in pkg");

    // Import with UpdateExisting mode, passing the module
    let mut thing_map = ThingMap::new();
    let updated_func = import_func(
        ctx,
        pkg_func,
        Some(module),
        &mut thing_map,
        false,
        UpdateExisting,
    )
    .await?;

    // Verify the func was updated (same ID, different code)
    assert_eq!(updated_func.id, func.id, "Expected same func ID");

    let func_code_after = updated_func
        .code_plaintext()?
        .expect("Expected func to have code");

    assert_ne!(
        func_code_before, func_code_after,
        "Expected code to be different after update"
    );
    assert_eq!(
        func_code_after, new_code,
        "Expected code to match the new code"
    );

    Ok(())
}
