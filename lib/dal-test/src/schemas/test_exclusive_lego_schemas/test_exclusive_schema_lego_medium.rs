use dal::action::prototype::ActionKind;
use dal::pkg::import_pkg_from_pkg;
use dal::ComponentType;
use dal::{BuiltinsResult, DalContext};
use si_pkg::SchemaSpecData;
use si_pkg::{
    ActionFuncSpec, PkgSpec, SchemaSpec, SchemaVariantSpec, SchemaVariantSpecData, SiPkg,
};

use crate::schemas::schema_helpers::{
    build_action_func, build_asset_func, build_resource_payload_to_value_func, create_identity_func,
};
use crate::schemas::test_exclusive_lego_schemas::bricks::LegoBricks;

pub(crate) async fn migrate_test_exclusive_schema_medium_odd_lego(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut medium_lego_builder = PkgSpec::builder();

    let schema_name = "medium odd lego";

    medium_lego_builder
        .name(schema_name)
        .version(crate::schemas::PKG_VERSION)
        .created_by(crate::schemas::PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionMediumLego";
    let create_action_func = build_action_func(create_action_code, fn_name).await?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionMediumLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name).await?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionMediumLego";
    let update_action_func = build_action_func(update_action_code, fn_name).await?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldMediumLegoAsset";
    let medium_lego_authoring_schema_func = build_asset_func(fn_name).await?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func().await?;

    let bricks = LegoBricks::new_for_odd()?;

    let medium_lego_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("medium_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&medium_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()
                        .expect("build variant spec data"),
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .domain_prop(bricks.domain_three_prop)
                .domain_prop(bricks.domain_four_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
                .socket(bricks.socket_three)
                .socket(bricks.socket_four)
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

    let medium_lego_spec = medium_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(medium_lego_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .schema(medium_lego_schema)
        .build()?;

    let medium_lego_pkg = SiPkg::load_from_spec(medium_lego_spec)?;
    import_pkg_from_pkg(ctx, &medium_lego_pkg, None).await?;

    Ok(())
}
pub(crate) async fn migrate_test_exclusive_schema_medium_even_lego(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut medium_lego_builder = PkgSpec::builder();

    let schema_name = "medium even lego";

    medium_lego_builder
        .name(schema_name)
        .version(crate::schemas::PKG_VERSION)
        .created_by(crate::schemas::PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionMediumLego";
    let create_action_func = build_action_func(create_action_code, fn_name).await?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionMediumLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name).await?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionMediumLego";
    let update_action_func = build_action_func(update_action_code, fn_name).await?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldMediumLegoAsset";
    let medium_lego_authoring_schema_func = build_asset_func(fn_name).await?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func().await?;

    let bricks = LegoBricks::new_for_even()?;

    let medium_lego_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("medium_even_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&medium_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()
                        .expect("build variant spec data"),
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .domain_prop(bricks.domain_three_prop)
                .domain_prop(bricks.domain_four_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
                .socket(bricks.socket_three)
                .socket(bricks.socket_four)
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

    let medium_lego_spec = medium_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(medium_lego_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .schema(medium_lego_schema)
        .build()?;

    let medium_lego_pkg = SiPkg::load_from_spec(medium_lego_spec)?;
    import_pkg_from_pkg(ctx, &medium_lego_pkg, None).await?;

    Ok(())
}
