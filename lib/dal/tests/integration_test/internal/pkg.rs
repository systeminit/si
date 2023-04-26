use base64::{engine::general_purpose, Engine};
use dal::{
    func::backend::validation::FuncBackendValidationArgs, installed_pkg::*, pkg::*,
    schema::variant::leaves::LeafKind, validation::Validation, DalContext, ExternalProvider, Func,
    InternalProvider, Schema, SchemaVariant, StandardModel, ValidationPrototype,
};
use dal_test::test;
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, LeafFunctionSpec,
    LeafInputLocation as PkgLeafInputLocation, LeafKind as PkgLeafKind, PkgSpec, PropSpec,
    PropSpecKind, SchemaSpec, SchemaVariantSpec, SiPkg, SocketSpec, SocketSpecArity,
    SocketSpecKind, ValidationSpec, ValidationSpecKind,
};

#[test]
async fn test_install_pkg(ctx: &DalContext) {
    let qualification_code = "function qualification(_input) { return { result: 'warning', message: 'omit needless words' }; } }";
    let qualification_b64 = general_purpose::STANDARD_NO_PAD.encode(qualification_code.as_bytes());

    let qualification_func_spec = FuncSpec::builder()
        .name("si:qualificationWarning")
        .display_name("warning")
        .description("it warns")
        .handler("qualification")
        .code_base64(&qualification_b64)
        .backend_kind(FuncSpecBackendKind::JsAttribute)
        .response_type(FuncSpecBackendResponseType::Qualification)
        .hidden(false)
        .build()
        .expect("build qual func spec");

    let qualification_spec = LeafFunctionSpec::builder()
        .func_unique_id(qualification_func_spec.unique_id)
        .leaf_kind(PkgLeafKind::Qualification)
        .inputs(vec![
            PkgLeafInputLocation::Domain,
            PkgLeafInputLocation::Code,
        ])
        .build()
        .expect("could not build qual spec");

    let schema_a = SchemaSpec::builder()
        .name("Tyrone Slothrop")
        .category("Banana Puddings")
        .variant(
            SchemaVariantSpec::builder()
                .name("Pig Bodine")
                .color("baddad")
                .prop(
                    PropSpec::builder()
                        .name("ImpolexG")
                        .kind(PropSpecKind::String)
                        .build()
                        .expect("able to make prop spec"),
                )
                .prop(
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
        .display_name("Horse Feathers")
        .description("it rejects values")
        .handler("validate")
        .code_base64(&validation_b64)
        .backend_kind(FuncSpecBackendKind::JsValidation)
        .response_type(FuncSpecBackendResponseType::Validation)
        .hidden(false)
        .build()
        .expect("able to build validation func spec");

    let schema_b = SchemaSpec::builder()
        .name("Roger Mexico")
        .category("Banana Puddings")
        .variant(
            SchemaVariantSpec::builder()
                .name("The Light Bulb Conspiracy")
                .color("baddad")
                .socket(
                    SocketSpec::builder()
                        .name("AC Power")
                        .kind(SocketSpecKind::Input)
                        .arity(SocketSpecArity::One)
                        .build()
                        .expect("able to make input socket"),
                )
                .socket(
                    SocketSpec::builder()
                        .name("Light")
                        .kind(SocketSpecKind::Output)
                        .arity(SocketSpecArity::Many)
                        .build()
                        .expect("able to make output socket"),
                )
                .prop(
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
                .prop(
                    PropSpec::builder()
                        .name("sixes_and_sevens")
                        .kind(PropSpecKind::Number)
                        .validation(
                            ValidationSpec::builder()
                                .kind(ValidationSpecKind::CustomValidation)
                                .func_unique_id(validation_func_spec.unique_id)
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

    let func_spec = FuncSpec::builder()
        .name("si:truthy")
        .display_name("truth")
        .description("it returns true")
        .handler("truth")
        .code_base64(&code_base64)
        .backend_kind(FuncSpecBackendKind::JsAttribute)
        .response_type(FuncSpecBackendResponseType::Boolean)
        .hidden(false)
        .build()
        .expect("build func spec");

    let func_spec_2 = FuncSpec::builder()
        .name("si:truthy")
        .display_name("truth")
        .description("it returns true")
        .handler("truth")
        .code_base64(&code_base64)
        .backend_kind(FuncSpecBackendKind::JsAttribute)
        .response_type(FuncSpecBackendResponseType::Boolean)
        .hidden(false)
        .build()
        .expect("build func spec");

    let func_spec_3 = FuncSpec::builder()
        .name("si:truthy")
        .display_name("truth")
        .description("it returns true, but this time with a different description")
        .handler("truth")
        .code_base64(&code_base64)
        .backend_kind(FuncSpecBackendKind::JsAttribute)
        .response_type(FuncSpecBackendResponseType::Boolean)
        .hidden(false)
        .build()
        .expect("build func spec");

    // Ensure unique ids are stable and change with properties changing
    assert_eq!(func_spec.unique_id, func_spec_2.unique_id);
    assert_ne!(func_spec.unique_id, func_spec_3.unique_id);

    let spec_a = PkgSpec::builder()
        .name("The White Visitation")
        .version("0.1")
        .created_by("Pirate Prentice")
        .schema(schema_a.clone())
        .func(func_spec)
        .func(qualification_func_spec)
        .build()
        .expect("able to build package spec");

    let pkg_a = SiPkg::load_from_spec(spec_a).expect("able to load from spec");

    let spec_b = PkgSpec::builder()
        .name("The Kenosha Kid")
        .version("0.1")
        .created_by("Pointsman")
        .func(validation_func_spec)
        .schema(schema_a)
        .schema(schema_b)
        .build()
        .expect("able to build package spec");

    let pkg_b = SiPkg::load_from_spec(spec_b).expect("able to load pkg from spec");

    import_pkg_from_pkg(ctx, &pkg_a, "pkg_a")
        .await
        .expect("able to install pkg");

    // We should refuse to install the same package twice
    let second_import_result = import_pkg_from_pkg(ctx, &pkg_a, "pkg_a").await;
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

    // One schema, one variant, two funcs
    assert_eq!(4, pkg_a_ipas.len());

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

    import_pkg_from_pkg(ctx, &pkg_b, "pkg_b")
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

    let pkg_b_ipas = InstalledPkgAsset::list_for_installed_pkg_id(ctx, *installed_pkg_b.id())
        .await
        .expect("able to fetch installed pkgs for pkg a");

    // Two schemas, two variants, one func
    assert_eq!(5, pkg_b_ipas.len());

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
