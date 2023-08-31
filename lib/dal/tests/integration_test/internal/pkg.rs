use base64::{engine::general_purpose, Engine};
use dal::{
    func::backend::validation::FuncBackendValidationArgs, func::intrinsics::IntrinsicFunc,
    installed_pkg::*, pkg::*, schema::variant::leaves::LeafKind, validation::Validation, ChangeSet,
    ChangeSetPk, DalContext, ExternalProvider, Func, InternalProvider, Schema, SchemaVariant,
    StandardModel, ValidationPrototype,
};
use dal_test::{test, DalContextHeadRef};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, LeafFunctionSpec,
    LeafInputLocation as PkgLeafInputLocation, LeafKind as PkgLeafKind, PkgSpec, PropSpec,
    PropSpecKind, SchemaSpec, SchemaSpecData, SchemaVariantSpec, SchemaVariantSpecData, SiPkg,
    SocketSpec, SocketSpecArity, SocketSpecData, SocketSpecKind, ValidationSpec,
    ValidationSpecKind,
};

#[test]
async fn test_workspace_pkg_export(DalContextHeadRef(ctx): DalContextHeadRef<'_>) {
    let new_change_set = ChangeSet::new(ctx, "cs1", None)
        .await
        .expect("can create change set");

    let cs_ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(new_change_set.pk));

    let mut func = Func::find_by_name(&cs_ctx, "test:refreshActionStarfield")
        .await
        .expect("able to get refreshActionStarfield")
        .expect("func exists");

    func.delete_by_id(&cs_ctx).await.expect("able to delete");

    cs_ctx.blocking_commit().await.expect("able to commit");

    let change_set_2 = ChangeSet::new(ctx, "cs2", None)
        .await
        .expect("can create change set");
    let cs2_ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(change_set_2.pk));

    let mut func = Func::find_by_name(&cs2_ctx, "test:refreshActionStarfield")
        .await
        .expect("able to get refreshActionStarfield")
        .expect("func exists");

    func.set_display_name(&cs2_ctx, Some("foo"))
        .await
        .expect("set display name");
    cs2_ctx.blocking_commit().await.expect("able to commit");

    let mut exporter = PkgExporter::new_workspace_exporter("workspace", "sally@systeminit.com");

    let package_bytes = exporter.export_as_bytes(ctx).await.expect("able to export");

    let pkg = SiPkg::load_from_bytes(package_bytes).expect("able to load from bytes");
    let spec = pkg.to_spec().await.expect("can convert to spec");

    assert_eq!(Some("head"), spec.default_change_set.as_deref());
    assert_eq!(3, spec.change_sets.len());

    let cs1 = spec.change_sets.get(2).expect("has second change set");

    assert_eq!("cs1", &cs1.name);
    assert_eq!(Some("head"), cs1.based_on_change_set.as_deref());

    assert_eq!(1, cs1.funcs.len());
    let refresh_func_in_changeset = cs1.funcs.get(0).expect("get first func");
    assert_eq!(
        "test:refreshActionStarfield",
        &refresh_func_in_changeset.name
    );
    assert!(matches!(refresh_func_in_changeset.data, None));
    assert!(refresh_func_in_changeset.deleted);

    let cs2 = spec.change_sets.get(1).expect("has second change set");

    assert_eq!("cs2", &cs2.name);
    assert_eq!(Some("head"), cs2.based_on_change_set.as_deref());

    assert_eq!(1, cs2.funcs.len());
    let refresh_func_in_changeset = cs2.funcs.get(0).expect("get first func");

    assert_eq!(
        Some("foo"),
        refresh_func_in_changeset
            .data
            .as_ref()
            .and_then(|data| data.display_name.as_deref())
    );
}

#[test]
async fn test_module_pkg_export(DalContextHeadRef(ctx): DalContextHeadRef<'_>) {
    let generic_frame_id = Schema::find_by_name(ctx, "Generic Frame")
        .await
        .expect("get generic frame")
        .id()
        .to_owned();

    let starfield_id = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("get starfield")
        .id()
        .to_owned();

    let fallout_id = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("get fallout")
        .id()
        .to_owned();

    let schema_ids = vec![generic_frame_id, starfield_id, fallout_id];

    let mut exporter = PkgExporter::new_module_exporter(
        "module",
        "test-version",
        None::<String>,
        "sally@systeminit.com",
        schema_ids,
    );

    let package_bytes = exporter.export_as_bytes(ctx).await.expect("able to export");

    let pkg = SiPkg::load_from_bytes(package_bytes).expect("able to load from bytes");
    let _spec = pkg.to_spec().await.expect("can convert to spec");

    let new_change_set = ChangeSet::new(ctx, "cs1", None)
        .await
        .expect("can create change set");

    let new_ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(new_change_set.pk));
    import_pkg_from_pkg(
        &new_ctx,
        &pkg,
        pkg.metadata().expect("get metadata").name(),
        None,
    )
    .await
    .expect("able to import pkg");

    let sv_change_sets: Vec<ChangeSetPk> = SchemaVariant::list(&new_ctx)
        .await
        .expect("get svs again")
        .iter()
        .map(|sv| sv.visibility().change_set_pk)
        .collect();

    let installed_variants = sv_change_sets
        .into_iter()
        .filter(|cspk| *cspk == new_change_set.pk)
        .collect::<Vec<ChangeSetPk>>();

    assert_eq!(3, installed_variants.len());

    let installed_schemas: Vec<Schema> = Schema::list(&new_ctx)
        .await
        .expect("get schemas")
        .into_iter()
        .filter(|schema| schema.visibility().change_set_pk == new_change_set.pk)
        .collect();

    let generic_frame = installed_schemas
        .iter()
        .find(|schema| schema.name() == "Generic Frame")
        .expect("able to find generic frame");

    dbg!(generic_frame
        .ui_menus(&new_ctx)
        .await
        .expect("get ui menus for generic frame"));
}

#[test]
async fn test_install_pkg(ctx: &DalContext) {
    let qualification_code = "function qualification(_input) { return { result: 'warning', message: 'omit needless words' }; } }";
    let qualification_b64 = general_purpose::STANDARD_NO_PAD.encode(qualification_code.as_bytes());

    let qualification_func_spec = FuncSpec::builder()
        .name("si:qualificationWarning")
        .unique_id("si:qualificationWarning")
        .data(
            FuncSpecData::builder()
                .name("si:qualificationWarning")
                .display_name("warning")
                .description("it warns")
                .handler("qualification")
                .code_base64(&qualification_b64)
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Qualification)
                .hidden(false)
                .build()
                .expect("able to build func data for qual"),
        )
        .build()
        .expect("build qual func spec");

    let qualification_spec = LeafFunctionSpec::builder()
        .func_unique_id(&qualification_func_spec.unique_id)
        .leaf_kind(PkgLeafKind::Qualification)
        .inputs(vec![
            PkgLeafInputLocation::Domain,
            PkgLeafInputLocation::Code,
        ])
        .build()
        .expect("could not build qual spec");

    let scaffold_func_a = "function createAsset() {
                return new AssetBuilder().build();
            }";
    let scaffold_func_spec_a = FuncSpec::builder()
        .name("si:scaffoldFuncA")
        .unique_id("si:scaffoldFuncA")
        .data(
            FuncSpecData::builder()
                .name("si:scaffoldFuncA")
                .code_plaintext(scaffold_func_a)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()
                .expect("build func data"),
        )
        .build()
        .expect("could not build schema variant definition spec");

    let schema_a = SchemaSpec::builder()
        .name("Tyrone Slothrop")
        .data(
            SchemaSpecData::builder()
                .name("Tyrone Slothrop")
                .category("Banana Puddings")
                .ui_hidden(false)
                .build()
                .expect("you never did the kenosha kid?"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("Pig Bodine")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("Pig Bodine")
                        .color("baddad")
                        .func_unique_id(&scaffold_func_spec_a.unique_id)
                        .build()
                        .expect("pig bodine"),
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("ImpolexG")
                        .kind(PropSpecKind::String)
                        .build()
                        .expect("able to make prop spec"),
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("TheZone")
                        .kind(PropSpecKind::String)
                        .build()
                        .expect("able to make prop spec"),
                )
                .leaf_function(qualification_spec)
                .build()
                .expect("able to make schema variant spec"),
        )
        .build()
        .expect("able to make schema spec");

    let custom_validation_code = "function validate(value) { return { valid: false, message: 'whatever it is, im against it' }; }";
    let validation_b64 = general_purpose::STANDARD_NO_PAD.encode(custom_validation_code.as_bytes());
    let validation_func_spec = FuncSpec::builder()
        .name("groucho")
        .unique_id("groucho")
        .data(
            FuncSpecData::builder()
                .name("groucho")
                .display_name("Horse Feathers")
                .description("it rejects values")
                .handler("validate")
                .code_base64(&validation_b64)
                .backend_kind(FuncSpecBackendKind::JsValidation)
                .response_type(FuncSpecBackendResponseType::Validation)
                .hidden(false)
                .build()
                .expect("whatever it is, i'm against it"),
        )
        .build()
        .expect("able to build validation func spec");

    let scaffold_func_b = "function createAsset() {
                return new AssetBuilder().build();
            }";
    let scaffold_func_spec_b = FuncSpec::builder()
        .name("si:scaffoldFuncB")
        .unique_id("si:scaffoldFuncB")
        .data(
            FuncSpecData::builder()
                .name("si:scaffoldFuncB")
                .code_plaintext(scaffold_func_b)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()
                .expect("scaffold b func data"),
        )
        .build()
        .expect("could not build schema variant definition spec");

    let schema_b = SchemaSpec::builder()
        .name("Roger Mexico")
        .data(
            SchemaSpecData::builder()
                .name("Roger Mexico")
                .ui_hidden(false)
                .category("Banana Puddings")
                .build()
                .expect("roger mexico data"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("The Light Bulb Conspiracy")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("The Light Bulb Conspiracy")
                        .color("baddad")
                        .func_unique_id(&scaffold_func_spec_b.unique_id)
                        .build()
                        .expect("light bulb spec data"),
                )
                .socket(
                    SocketSpec::builder()
                        .name("AC Power")
                        .data(
                            SocketSpecData::builder()
                                .name("AC Power")
                                .ui_hidden(false)
                                .kind(SocketSpecKind::Input)
                                .arity(SocketSpecArity::One)
                                .build()
                                .expect("build socket data"),
                        )
                        .build()
                        .expect("able to make input socket"),
                )
                .socket(
                    SocketSpec::builder()
                        .name("Light")
                        .data(
                            SocketSpecData::builder()
                                .name("Light")
                                .kind(SocketSpecKind::Output)
                                .arity(SocketSpecArity::Many)
                                .ui_hidden(false)
                                .build()
                                .expect("build light socket data"),
                        )
                        .build()
                        .expect("able to make output socket"),
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("distress_jess")
                        .kind(PropSpecKind::Number)
                        .validation(
                            ValidationSpec::builder()
                                .kind(ValidationSpecKind::IntegerIsBetweenTwoIntegers)
                                .lower_bound(2)
                                .upper_bound(100)
                                .build()
                                .expect("able to add validation"),
                        )
                        .build()
                        .expect("able to make prop spec"),
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("sixes_and_sevens")
                        .kind(PropSpecKind::Number)
                        .validation(
                            ValidationSpec::builder()
                                .kind(ValidationSpecKind::CustomValidation)
                                .func_unique_id(&validation_func_spec.unique_id)
                                .build()
                                .expect("able to add custom validation"),
                        )
                        .build()
                        .expect("able to make prop spec"),
                )
                .build()
                .expect("able to make schema variant spec"),
        )
        .build()
        .expect("able to make schema spec");

    let code = "function truth() { return true; }";
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode(code.as_bytes());
    let identity_func_spec = IntrinsicFunc::Identity
        .to_spec()
        .expect("create identity func spec");

    let func_spec = FuncSpec::builder()
        .name("si:truthy")
        .unique_id("si:truthy")
        .data(
            FuncSpecData::builder()
                .name("si:truthy")
                .display_name("truth")
                .description("it returns true")
                .handler("truth")
                .code_base64(&code_base64)
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Boolean)
                .hidden(false)
                .build()
                .expect("truth func data"),
        )
        .build()
        .expect("build func spec");

    let spec_a = PkgSpec::builder()
        .name("The White Visitation")
        .version("0.1")
        .created_by("Pirate Prentice")
        .schema(schema_a.clone())
        .func(func_spec)
        .func(identity_func_spec.clone())
        .func(qualification_func_spec)
        .func(scaffold_func_spec_a.clone())
        .build()
        .expect("able to build package spec");

    let pkg_a = SiPkg::load_from_spec(spec_a).expect("able to load from spec");

    let spec_b = PkgSpec::builder()
        .name("The Kenosha Kid")
        .version("0.1")
        .created_by("Pointsman")
        .func(validation_func_spec)
        .func(identity_func_spec.clone())
        .schema(schema_a)
        .schema(schema_b)
        .func(scaffold_func_spec_a)
        .func(scaffold_func_spec_b)
        .build()
        .expect("able to build package spec");

    let pkg_b = SiPkg::load_from_spec(spec_b).expect("able to load pkg from spec");

    import_pkg_from_pkg(ctx, &pkg_a, "pkg_a", None)
        .await
        .expect("able to install pkg");

    // We should refuse to install the same package twice
    let second_import_result = import_pkg_from_pkg(ctx, &pkg_a, "pkg_a", None).await;
    assert!(matches!(
        second_import_result,
        Err(PkgError::PackageAlreadyInstalled(_))
    ));

    // for now just assert that the installed package records are written. we should also verify
    // the structure of the installed schemas but punting on that for the moment
    let pkg_a_root_hash = pkg_a.hash().expect("pkg a has a hash").to_string();
    let installed_pkgs = InstalledPkg::find_by_attr(ctx, "root_hash", &pkg_a_root_hash)
        .await
        .expect("find by attr");
    assert_eq!(1, installed_pkgs.len());
    let installed_pkg_a = installed_pkgs.get(0).expect("pkg should be there");

    assert_eq!("pkg_a", installed_pkg_a.name());

    let pkg_a_ipas = InstalledPkgAsset::list_for_installed_pkg_id(ctx, *installed_pkg_a.id())
        .await
        .expect("able to fetch installed pkgs for pkg a");

    for ipa in pkg_a_ipas {
        match ipa.asset_kind() {
            InstalledPkgAssetKind::Schema => {
                let typed: InstalledPkgAssetTyped =
                    ipa.as_installed_schema().expect("get schema ipa type");
                match typed {
                    InstalledPkgAssetTyped::Schema { id, .. } => {
                        let schema = Schema::get_by_id(ctx, &id)
                            .await
                            .expect("able to get schema")
                            .expect("schema is there");

                        assert_eq!("Tyrone Slothrop", schema.name())
                    }
                    _ => unreachable!(),
                }
            }
            InstalledPkgAssetKind::SchemaVariant => {
                let typed: InstalledPkgAssetTyped = ipa
                    .as_installed_schema_variant()
                    .expect("get schema ipa type");
                match typed {
                    InstalledPkgAssetTyped::SchemaVariant { id, .. } => {
                        let schema_variant = SchemaVariant::get_by_id(ctx, &id)
                            .await
                            .expect("able to get schema variant")
                            .expect("schema variant is there");

                        assert_eq!("Pig Bodine", schema_variant.name());

                        let qualifications = SchemaVariant::find_leaf_item_functions(
                            ctx,
                            *schema_variant.id(),
                            LeafKind::Qualification,
                        )
                        .await
                        .expect("able to get qualification funcs");

                        assert_eq!(1, qualifications.len())
                    }
                    _ => unreachable!(),
                }
            }
            InstalledPkgAssetKind::SchemaVariantDefinition => {}
            InstalledPkgAssetKind::Func => {
                let typed: InstalledPkgAssetTyped =
                    ipa.as_installed_func().expect("get func ipa typed");
                match typed {
                    InstalledPkgAssetTyped::Func { id, .. } => {
                        let _func = Func::get_by_id(ctx, &id)
                            .await
                            .expect("able to get func")
                            .expect("func is there");
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    import_pkg_from_pkg(ctx, &pkg_b, "pkg_b", None)
        .await
        .expect("install pkg b");

    let installed_pkgs = InstalledPkg::find_by_attr(
        ctx,
        "root_hash",
        &pkg_b.hash().expect("get pkg b hash").to_string(),
    )
    .await
    .expect("find by attr");
    assert_eq!(1, installed_pkgs.len());
    let installed_pkg_b = installed_pkgs.get(0).expect("pkg should be there");

    assert_eq!("pkg_b", installed_pkg_b.name());

    let _pkg_b_ipas = InstalledPkgAsset::list_for_installed_pkg_id(ctx, *installed_pkg_b.id())
        .await
        .expect("able to fetch installed pkgs for pkg a");

    // Ensure we did not install the schema that is in both packages twice
    let schemas = Schema::find_by_attr(ctx, "name", &"Tyrone Slothrop".to_string())
        .await
        .expect("get tyrones");
    assert_eq!(1, schemas.len());
    let schema_variants = SchemaVariant::find_by_attr(ctx, "name", &"Pig Bodine".to_string())
        .await
        .expect("get bodines");
    assert_eq!(1, schema_variants.len());

    // Ensure the new schemas were installed
    let schemas = Schema::find_by_attr(ctx, "name", &"Roger Mexico".to_string())
        .await
        .expect("roger roger roger roger");
    assert_eq!(1, schemas.len());
    let mut schema_variants =
        SchemaVariant::find_by_attr(ctx, "name", &"The Light Bulb Conspiracy".to_string())
            .await
            .expect("above your head like you had an idea");
    assert_eq!(1, schema_variants.len());

    let light_bulb = schema_variants
        .pop()
        .expect("able to get the light bulb conspiracy");

    let _ac_input = InternalProvider::find_explicit_for_schema_variant_and_name(
        ctx,
        *light_bulb.id(),
        "AC Power",
    )
    .await
    .expect("able to search for ac input")
    .expect("able to find ac input");

    let _light_output =
        ExternalProvider::find_for_schema_variant_and_name(ctx, *light_bulb.id(), "Light")
            .await
            .expect("able to search for light output")
            .expect("able to find light output");

    let validations = ValidationPrototype::list_for_schema_variant(ctx, *light_bulb.id())
        .await
        .expect("able to find validations");

    // The "/root/color" prop gets a StringIsHexColor validation, plus the validation from the spec
    assert_eq!(3, validations.len());
    let number_validation = validations.get(1).expect("able to get validation");
    let validation_args: FuncBackendValidationArgs =
        serde_json::from_value(number_validation.args().clone()).expect("able to deserialize");

    assert!(matches!(
        validation_args.validation,
        Validation::IntegerIsBetweenTwoIntegers {
            lower_bound: 2,
            upper_bound: 100,
            ..
        }
    ));

    let custom_validation = validations.get(2).expect("able to get custom validation");
    let func = Func::get_by_id(ctx, &custom_validation.func_id())
        .await
        .expect("able to get func")
        .expect("func is there");
    assert_eq!(func.name(), "groucho");
}
