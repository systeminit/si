use base64::{engine::general_purpose, Engine};
use dal::{
    func::{
        argument::FuncArgumentKind, backend::validation::FuncBackendValidationArgs,
        intrinsics::IntrinsicFunc,
    },
    installed_pkg::*,
    pkg::*,
    prop::PropPath,
    schema::variant::leaves::LeafKind,
    validation::Validation,
    ActionKind, ChangeSet, ChangeSetPk, DalContext, ExternalProvider, Func, InternalProvider,
    PropKind, Schema, SchemaVariant, StandardModel, ValidationPrototype,
};
use dal::{BuiltinsResult, ComponentType};
use dal_test::{test, DalContextHeadRef};
use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, LeafFunctionSpec,
    LeafInputLocation as PkgLeafInputLocation, LeafKind as PkgLeafKind, PkgSpec, PropSpec,
    PropSpecKind, SchemaSpec, SchemaSpecData, SchemaVariantSpec, SchemaVariantSpecData, SiPkg,
    SocketSpec, SocketSpecArity, SocketSpecData, SocketSpecKind, ValidationSpec,
    ValidationSpecKind,
};

async fn make_stellarfield(ctx: &DalContext) -> BuiltinsResult<()> {
    let mut stellarfield_builder = PkgSpec::builder();

    stellarfield_builder
        .name("stellarfield")
        .version("2023-05-23")
        .created_by("System Initiative");

    let identity_func_spec = IntrinsicFunc::Identity
        .to_spec()
        .expect("create identity func spec");

    let stellarfield_create_action_code = "async function create() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionStellarfield";
    let stellarfield_create_action_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(stellarfield_create_action_code)
                .handler("create")
                .backend_kind(FuncSpecBackendKind::JsAction)
                .response_type(FuncSpecBackendResponseType::Action)
                .build()?,
        )
        .build()?;

    let stellarfield_refresh_action_code =
        "async function refresh(component: Input): Promise<Output> {
              return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:refreshActionStellarfield";
    let stellarfield_refresh_action_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .handler("refresh")
                .code_plaintext(stellarfield_refresh_action_code)
                .backend_kind(FuncSpecBackendKind::JsAction)
                .response_type(FuncSpecBackendResponseType::Action)
                .build()?,
        )
        .build()?;

    let fallout_entries_to_galaxies_transform_code =
        "async function falloutEntriesToGalaxies(input: Input): Promise<Output> {
          let galaxies = [];
          let entries = input.entries;

          // Force the entries arg to be an Array (and return an empty array if the arg is absent/undefined/null).
          if (entries === undefined) return galaxies;
          if (entries === null) return galaxies;
          if (!Array.isArray(entries)) entries = [entries];

          entries.filter(i => i ?? false).forEach(function (entry) {

            let name = entry.si.name;
            let rads = entry.domain.rads;
            let galaxy = {
              sun: name + \"-sun\",
              planets: rads
            };

            galaxies.push(galaxy);
          });

          return galaxies;
        }";
    let fn_name = "test:falloutEntriesToGalaxiesStellarfield";
    let fallout_entries_to_galaxies_transform_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fallout_entries_to_galaxies_transform_code)
                .handler("falloutEntriesToGalaxies")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Array)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("entries")
                .kind(FuncArgumentKind::Array)
                .element_kind(Some(FuncArgumentKind::Object.into()))
                .build()?,
        )
        .build()?;

    let stellarfield_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldStellarfieldAsset";
    let stellarfield_authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(stellarfield_scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let stellarfield_resource_payload_to_value_func_code =
        "async function translate(arg: Input): Promise<Output> {\
            return arg.payload ?? {};
        }";
    let fn_name = "test:resourcePayloadToValue";
    let stellarfield_resource_payload_to_value_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(stellarfield_resource_payload_to_value_func_code)
                .handler("translate")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Json)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("payload")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .build()?;

    let stellarfield_schema = SchemaSpec::builder()
        .name("stellarfield")
        .data(
            SchemaSpecData::builder()
                .name("stellarfield")
                .category("test exclusive")
                .category_name("stellarfield")
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("stellarfield_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&stellarfield_authoring_schema_func.unique_id)
                        .build()
                        .expect("build variant spec data"),
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("name")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::Prop)
                                .name("identity")
                                .prop_path(PropPath::new(["root", "si", "name"]))
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("hidden_prop")
                        .kind(PropKind::String)
                        .hidden(true)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("freestar")
                        .kind(PropKind::String)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("attributes")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("bethesda")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("universe")
                        .kind(PropKind::Object)
                        .entry(
                            PropSpec::builder()
                                .name("galaxies")
                                .kind(PropKind::Array)
                                .func_unique_id(
                                    &fallout_entries_to_galaxies_transform_func.unique_id,
                                )
                                .input(
                                    AttrFuncInputSpec::builder()
                                        .kind(AttrFuncInputSpecKind::InputSocket)
                                        .name("entries")
                                        .socket_name("fallout")
                                        .build()?,
                                )
                                .type_prop(
                                    PropSpec::builder()
                                        .name("galaxy")
                                        .kind(PropKind::Object)
                                        .entry(
                                            PropSpec::builder()
                                                .name("sun")
                                                .kind(PropKind::String)
                                                .build()?,
                                        )
                                        .entry(
                                            PropSpec::builder()
                                                .name("planets")
                                                .kind(PropKind::Integer)
                                                .build()?,
                                        )
                                        .build()?,
                                )
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("bethesda")
                        .data(
                            SocketSpecData::builder()
                                .name("bethesda")
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("fallout")
                        .data(
                            SocketSpecData::builder()
                                .name("fallout")
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Create)
                        .func_unique_id(&stellarfield_create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Refresh)
                        .func_unique_id(&stellarfield_refresh_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let stellarfield_spec = stellarfield_builder
        .func(identity_func_spec)
        .func(stellarfield_refresh_action_func)
        .func(stellarfield_create_action_func)
        .func(fallout_entries_to_galaxies_transform_func)
        .func(stellarfield_authoring_schema_func)
        .func(stellarfield_resource_payload_to_value_func)
        .schema(stellarfield_schema)
        .build()?;

    let stellarfield_pkg = SiPkg::load_from_spec(stellarfield_spec)?;
    import_pkg_from_pkg(
        ctx,
        &stellarfield_pkg,
        Some(dal::pkg::ImportOptions {
            schemas: Some(vec!["stellarfield".into()]),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

#[test]
async fn test_workspace_pkg_export(DalContextHeadRef(ctx): DalContextHeadRef<'_>) {
    let new_change_set = ChangeSet::new(ctx, "cs1", None)
        .await
        .expect("can create change set");

    let cs_ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(new_change_set.pk));

    make_stellarfield(&cs_ctx)
        .await
        .expect("able to make stellarfield");

    let mut exporter =
        PkgExporter::new_workspace_exporter("workspace", "sally@systeminit.com", "foo", "bar");

    let package_bytes = exporter.export_as_bytes(ctx).await.expect("able to export");

    let pkg = SiPkg::load_from_bytes(package_bytes).expect("able to load from bytes");
    let _spec = pkg.to_spec().await.expect("can convert to spec");

    import_pkg_from_pkg(ctx, &pkg, None)
        .await
        .expect("able to import workspace");
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
    import_pkg_from_pkg(&new_ctx, &pkg, None)
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
                        .component_type(ComponentType::Component)
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

    import_pkg_from_pkg(ctx, &pkg_a, None)
        .await
        .expect("able to install pkg");

    // We should refuse to install the same package twice
    let second_import_result = import_pkg_from_pkg(ctx, &pkg_a, None).await;
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

    assert_eq!("The White Visitation", installed_pkg_a.name());

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

    import_pkg_from_pkg(ctx, &pkg_b, None)
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

    assert_eq!("The Kenosha Kid", installed_pkg_b.name());

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
