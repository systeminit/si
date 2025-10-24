use dal::{
    BuiltinsResult,
    ComponentType,
    DalContext,
    PropKind,
    SchemaId,
    action::prototype::ActionKind,
    pkg::{
        ImportOptions,
        import_pkg_from_pkg,
    },
    prop::PropPath,
};
use si_pkg::{
    ActionFuncSpec,
    AttrFuncInputSpec,
    AttrFuncInputSpecKind,
    FuncSpec,
    FuncSpecBackendKind,
    FuncSpecBackendResponseType,
    FuncSpecData,
    LeafFunctionSpec,
    LeafInputLocation,
    LeafKind,
    PkgSpec,
    PropSpec,
    SchemaSpec,
    SchemaSpecData,
    SchemaVariantSpec,
    SchemaVariantSpecData,
    SiPkg,
    SocketSpec,
    SocketSpecData,
    SocketSpecKind,
};

use crate::test_exclusive_schemas::{
    PKG_CREATED_BY,
    PKG_VERSION,
    build_action_func,
    build_asset_func,
    build_codegen_func,
    create_identity_func,
};

pub(crate) async fn migrate_test_exclusive_schema_swifty(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut swifty_builder = PkgSpec::builder();

    let schema_name = "swifty";

    swifty_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionSwifty";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Update Action Func
    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionSwifty";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Build Delete Action Func
    let delete_action_code = "async function main() {
                return { payload: undefined, status: \"ok\" };
            }";

    let fn_name = "test:deleteActionSwifty";
    let delete_action_func = build_action_func(delete_action_code, fn_name)?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionSwifty";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldSwiftyAsset";
    let swifty_authoring_schema_func = build_asset_func(fn_name)?;

    // Build CodeGen Func
    let codegen_fn_name = "test:generateCode";
    let codegen_func_code = "async function main(input: Input): Promise < Output > {
                return {
                    format: \"json\",
                    code: JSON.stringify(input.domain || {}, null, 2),
                };
            }";
    let code_gen_func = build_codegen_func(codegen_func_code, codegen_fn_name)?;

    // Assemble Qualification Func
    let (qualification_func_spec, qualfiication_leaf_func_spec) = assemble_qualification()?;

    let swifty_schema = SchemaSpec::builder()
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
                .unique_id("swifty_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&swifty_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
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
                .socket(
                    SocketSpec::builder()
                        .name("fallout")
                        .data(
                            SocketSpecData::builder()
                                .name("fallout")
                                .connection_annotations(serde_json::to_string(&vec!["fallout"])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("anything")
                        .data(
                            SocketSpecData::builder()
                                .name("anything")
                                .connection_annotations(serde_json::to_string(&vec!["anything"])?)
                                .kind(SocketSpecKind::Output)
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Create)
                        .name(Some("test:createActionSwifty".to_string()))
                        .func_unique_id(&create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Destroy)
                        .func_unique_id(&delete_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Refresh)
                        .func_unique_id(&refresh_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Update)
                        .func_unique_id(&update_action_func.unique_id)
                        .build()?,
                )
                .leaf_function(
                    LeafFunctionSpec::builder()
                        .func_unique_id(codegen_fn_name)
                        .leaf_kind(LeafKind::CodeGeneration)
                        .inputs(vec![LeafInputLocation::Domain])
                        .build()?,
                )
                .leaf_function(qualfiication_leaf_func_spec)
                .build()?,
        )
        .build()?;

    let swifty_spec = swifty_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(delete_action_func)
        .func(update_action_func)
        .func(swifty_authoring_schema_func)
        .func(code_gen_func)
        .func(qualification_func_spec)
        .schema(swifty_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(swifty_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

fn assemble_qualification() -> BuiltinsResult<(FuncSpec, LeafFunctionSpec)> {
    let fn_code = "async function qualification(_component: Input): Promise<Output> {\
        return {
            result: 'success',
            message: 'this cannot fail'
        };
    }";
    let fn_name = "test:swiftyQualification";
    let func_spec = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fn_code)
                .handler("qualification")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Qualification)
                .build()?,
        )
        .build()?;

    let leaf_function_spec = LeafFunctionSpec::builder()
        .func_unique_id(&func_spec.unique_id)
        .leaf_kind(LeafKind::Qualification)
        .inputs(vec![LeafInputLocation::Domain])
        .build()?;

    Ok((func_spec, leaf_function_spec))
}
