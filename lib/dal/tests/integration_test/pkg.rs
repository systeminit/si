use base64::{engine::general_purpose, Engine};
use dal::{installed_pkg::*, pkg::*, DalContext, Func, Schema, SchemaVariant, StandardModel};
use dal_test::test;
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, PkgSpec, PropSpec, PropSpecKind,
    SchemaSpec, SchemaVariantSpec, SiPkg,
};

#[test]
async fn test_install_pkg(ctx: &DalContext) {
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
                .build()
                .expect("able to make schema variant spec"),
        )
        .build()
        .expect("able to make schema spec");

    let schema_b = SchemaSpec::builder()
        .name("Roger Mexico")
        .category("Banana Puddings")
        .variant(
            SchemaVariantSpec::builder()
                .name("The Light Bulb Conspiracy")
                .color("baddad")
                .prop(
                    PropSpec::builder()
                        .name("distress_jess")
                        .kind(PropSpecKind::Number)
                        .build()
                        .expect("able to make prop spec"),
                )
                .prop(
                    PropSpec::builder()
                        .name("sixes_and_sevens")
                        .kind(PropSpecKind::Number)
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
        .build()
        .expect("able to build package spec");

    let pkg_a = SiPkg::load_from_spec(spec_a).expect("able to load from spec");

    let spec_b = PkgSpec::builder()
        .name("The Kenosha Kid")
        .version("0.1")
        .created_by("Pointsman")
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

    // One schema, one variant, one func
    assert_eq!(3, pkg_a_ipas.len());

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

                        assert_eq!("Pig Bodine", schema_variant.name())
                    }
                    _ => unreachable!(),
                }
            }
            InstalledPkgAssetKind::Func => {
                let typed: InstalledPkgAssetTyped =
                    ipa.as_installed_func().expect("get func ipa typed");
                match typed {
                    InstalledPkgAssetTyped::Func { id, .. } => {
                        let func = Func::get_by_id(ctx, &id)
                            .await
                            .expect("able to get func")
                            .expect("func is there");
                        assert_eq!("si:truthy", func.name());
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

    // Two schemas, two variants
    assert_eq!(4, pkg_b_ipas.len());

    // Ensure we did not install the schema that is in both packages
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
    let schema_variants =
        SchemaVariant::find_by_attr(ctx, "name", &"The Light Bulb Conspiracy".to_string())
            .await
            .expect("above your head like you had an idea");
    assert_eq!(1, schema_variants.len());
}
