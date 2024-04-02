use crate::schemas::schema_helpers::{
    build_action_func, build_asset_func, build_codegen_func, create_identity_func,
};
use dal::func::intrinsics::IntrinsicFunc;
use dal::pkg::import_pkg_from_pkg;
use dal::{pkg, prop::PropPath, ActionKind, ComponentType};
use dal::{BuiltinsResult, DalContext, PropKind};
use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, LeafInputLocation, LeafKind, PkgSpec,
    PropSpec, SchemaSpec, SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpec,
    SocketSpecData, SocketSpecKind,
};
use si_pkg::{LeafFunctionSpec, SchemaSpecData};

pub async fn migrate_test_exclusive_schema_swifty(ctx: &DalContext) -> BuiltinsResult<()> {
    let mut swifty_builder = PkgSpec::builder();

    swifty_builder
        .name("swifty")
        .version(crate::schemas::PKG_VERSION)
        .created_by(crate::schemas::PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionSwifty";
    let create_action_func = build_action_func(create_action_code, fn_name).await?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionSwifty";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name).await?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionSwifty";
    let update_action_func = build_action_func(update_action_code, fn_name).await?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldSwiftyAsset";
    let swifty_authoring_schema_func = build_asset_func(fn_name).await?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = IntrinsicFunc::ResourcePayloadToValue.to_spec()?;

    // Build CodeGen Func
    let codegen_fn_name = "test:generateCode";
    let codegen_func_code = "async function main(input: Input): Promise < Output > {
                return {
                    format: \"json\",
                    code: JSON.stringify(input.domain || {}, null, 2),
                };
            }";
    let code_gen_func = build_codegen_func(codegen_func_code, codegen_fn_name).await?;

    let swifty_schema = SchemaSpec::builder()
        .name("swifty")
        .data(
            SchemaSpecData::builder()
                .name("swifty")
                .category("test exclusive")
                .category_name("swifty")
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("swifty_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&swifty_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
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
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Refresh)
                        .func_unique_id(&refresh_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Other)
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
                .build()?,
        )
        .build()?;

    let swifty_spec = swifty_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(swifty_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .func(code_gen_func)
        .schema(swifty_schema)
        .build()?;

    let swifty_pkg = SiPkg::load_from_spec(swifty_spec)?;
    import_pkg_from_pkg(
        ctx,
        &swifty_pkg,
        Some(pkg::ImportOptions {
            schemas: Some(vec!["swifty".into()]),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
