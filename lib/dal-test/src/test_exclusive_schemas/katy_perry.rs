use dal::pkg::{import_pkg_from_pkg, ImportOptions};
use dal::{prop::PropPath, ComponentType};
use dal::{BuiltinsResult, DalContext, PropKind, SchemaId};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, LeafInputLocation, LeafKind, PkgSpec, PropSpec,
    SchemaSpec, SchemaVariantSpec, SchemaVariantSpecData, SiPkg,
};
use si_pkg::{LeafFunctionSpec, SchemaSpecData};

use crate::test_exclusive_schemas::{
    build_asset_func, build_codegen_func, create_identity_func, PKG_CREATED_BY, PKG_VERSION,
};

pub(crate) async fn migrate_test_exclusive_schema_katy_perry(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut kp_builder = PkgSpec::builder();

    let schema_name = "katy perry";

    kp_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldKatyPerryAsset";
    let kp_authoring_schema_func = build_asset_func(fn_name)?;

    // Build YAML CodeGen Func
    let yaml_codegen_fn_name = "test:generateYamlCode";
    let yaml_codegen_func_code = "async function main(input: Input): Promise < Output > {
                return {
                    format: \"yaml\",
                    code: Object.keys(input.domain).length > 0 ? YAML.stringify(input.domain) : \"\"
                };
            }";
    let yaml_code_gen_func = build_codegen_func(yaml_codegen_func_code, yaml_codegen_fn_name)?;

    // Build string CodeGen Func
    let string_codegen_fn_name = "test:generateStringCode";
    let string_codegen_func_code = "async function main(input: Input): Promise < Output > {
                return {
                    format: \"string\",
                    code: \"poop canoe\"
                };
            }";
    let string_code_gen_func =
        build_codegen_func(string_codegen_func_code, string_codegen_fn_name)?;

    let kp_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("katy_perry_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&kp_authoring_schema_func.unique_id)
                        .component_type(ComponentType::Component)
                        .build()?,
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
                .leaf_function(
                    LeafFunctionSpec::builder()
                        .func_unique_id(yaml_codegen_fn_name)
                        .leaf_kind(LeafKind::CodeGeneration)
                        .inputs(vec![LeafInputLocation::Domain])
                        .build()?,
                )
                .leaf_function(
                    LeafFunctionSpec::builder()
                        .func_unique_id(string_codegen_fn_name)
                        .leaf_kind(LeafKind::CodeGeneration)
                        .inputs(vec![LeafInputLocation::Domain])
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let kp_spec = kp_builder
        .func(identity_func_spec)
        .func(yaml_code_gen_func)
        .func(string_code_gen_func)
        .func(kp_authoring_schema_func)
        .schema(kp_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(kp_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
