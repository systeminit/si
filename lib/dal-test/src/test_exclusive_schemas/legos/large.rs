use dal::action::prototype::ActionKind;
use dal::pkg::import_pkg_from_pkg;
use dal::ComponentType;
use dal::{BuiltinsResult, DalContext};
use si_pkg::{
    ActionFuncSpec, PkgSpec, SchemaSpec, SchemaSpecData, SchemaVariantSpec, SchemaVariantSpecData,
    SiPkg,
};

use crate::test_exclusive_schemas::legos::bricks::LegoBricks;
use crate::test_exclusive_schemas::{
    build_action_func, build_asset_func, build_resource_payload_to_value_func,
    create_identity_func, PKG_CREATED_BY, PKG_VERSION,
};

pub(crate) async fn migrate_test_exclusive_schema_large_odd_lego(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut large_lego_builder = PkgSpec::builder();

    let schema_name = "large odd lego";

    large_lego_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionLargeLego";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionLargeLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionLargeLego";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldLargeLegoAsset";
    let large_lego_authoring_schema_func = build_asset_func(fn_name)?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func()?;

    let bricks = LegoBricks::new_for_odd()?;

    let large_lego_schema = SchemaSpec::builder()
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
                .unique_id("large_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&large_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()?,
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .domain_prop(bricks.domain_three_prop)
                .domain_prop(bricks.domain_four_prop)
                .domain_prop(bricks.domain_five_prop)
                .domain_prop(bricks.domain_six_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
                .socket(bricks.socket_three)
                .socket(bricks.socket_four)
                .socket(bricks.socket_five)
                .socket(bricks.socket_six)
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
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
                        .kind(ActionKind::Manual)
                        .func_unique_id(&update_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let large_lego_spec = large_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(large_lego_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .schema(large_lego_schema)
        .build()?;

    let large_lego_pkg = SiPkg::load_from_spec(large_lego_spec)?;
    import_pkg_from_pkg(ctx, &large_lego_pkg, None).await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_large_even_lego(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut large_lego_builder = PkgSpec::builder();

    let schema_name = "large even lego";

    large_lego_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionLargeLego";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionLargeLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionLargeLego";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldLargeLegoAsset";
    let large_lego_authoring_schema_func = build_asset_func(fn_name)?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func()?;

    let bricks = LegoBricks::new_for_even()?;

    let large_lego_schema = SchemaSpec::builder()
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
                .unique_id("large_even_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&large_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()?,
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .domain_prop(bricks.domain_three_prop)
                .domain_prop(bricks.domain_four_prop)
                .domain_prop(bricks.domain_five_prop)
                .domain_prop(bricks.domain_six_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
                .socket(bricks.socket_three)
                .socket(bricks.socket_four)
                .socket(bricks.socket_five)
                .socket(bricks.socket_six)
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
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
                        .kind(ActionKind::Manual)
                        .func_unique_id(&update_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let large_lego_spec = large_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(large_lego_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .schema(large_lego_schema)
        .build()?;

    let large_lego_pkg = SiPkg::load_from_spec(large_lego_spec)?;
    import_pkg_from_pkg(ctx, &large_lego_pkg, None).await?;

    Ok(())
}
