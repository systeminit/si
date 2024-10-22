use dal::action::prototype::ActionKind;
use dal::pkg::{import_pkg_from_pkg, ImportOptions};
use dal::{BuiltinsResult, DalContext};
use dal::{ComponentType, SchemaId};
use si_pkg::{
    ActionFuncSpec, PkgSpec, SchemaSpec, SchemaVariantSpec, SchemaVariantSpecData, SiPkg,
};
use si_pkg::{ManagementFuncSpec, SchemaSpecData};

use crate::test_exclusive_schemas::legos::bricks::LegoBricks;
use crate::test_exclusive_schemas::{
    build_action_func, build_asset_func, build_management_func,
    build_resource_payload_to_value_func, create_identity_func, PKG_CREATED_BY, PKG_VERSION,
};

pub(crate) async fn migrate_test_exclusive_schema_small_odd_lego(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut small_lego_builder = PkgSpec::builder();

    let schema_name = "small odd lego";

    small_lego_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionSmallLego";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
        return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
    }";

    let fn_name = "test:refreshActionSmallLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    // Build Update Action Func
    let update_action_code = "async function main(component: Input): Promise<Output> {
        return { payload: { \"poonami\": true }, status: \"ok\" };
    }";
    let fn_name = "test:updateActionSmallLego";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Build Delete Action Func
    let delete_action_code = "async function main() {
        return { payload: null, status: \"ok\" };
    }";

    let import_management_func_code =
        "async function main({ thisComponent }: Input): Promise<Output> {
        const thisProperties = thisComponent.properties;
        return { 
            status: 'ok',
            ops: {
                update: {
                    self: {
                        properties: {
                            domain: {
                                ...thisProperties.domain
                                two: 'step',
                            }
                            ...thisProperties
                        }
                    }
                }
            },
            message: 'hello'
        }
    }";
    let import_management_func_name = "test:importManagementSmallLego";
    let import_management_func =
        build_management_func(import_management_func_code, import_management_func_name)?;

    let fn_name = "test:deleteActionSmallLego";
    let delete_action_func = build_action_func(delete_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldSmallLegoAsset";
    let small_lego_authoring_schema_func = build_asset_func(fn_name)?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func()?;

    let bricks = LegoBricks::new_for_odd()?;

    let small_lego_schema = SchemaSpec::builder()
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
                .unique_id("small_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&small_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()?,
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
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
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Destroy)
                        .func_unique_id(&delete_action_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Import small odd lego")
                        .func_unique_id(&import_management_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let small_lego_spec = small_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(delete_action_func)
        .func(small_lego_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .func(import_management_func)
        .schema(small_lego_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(small_lego_spec)?;
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

pub(crate) async fn migrate_test_exclusive_schema_small_even_lego(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut small_lego_builder = PkgSpec::builder();

    let schema_name = "small even lego";

    small_lego_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionSmallLego";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
            }";

    let fn_name = "test:refreshActionSmallLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poonami\": true }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionSmallLego";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Build Delete Action Func
    let delete_action_code = "async function main() {
        return { payload: null, status: \"ok\" };
    }";

    let fn_name = "test:deleteActionSmallLego";
    let delete_action_func = build_action_func(delete_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldSmallLegoAsset";
    let small_lego_authoring_schema_func = build_asset_func(fn_name)?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func()?;

    let bricks = LegoBricks::new_for_even()?;

    let small_lego_schema = SchemaSpec::builder()
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
                .unique_id("small_even_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&small_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()?,
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
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
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Destroy)
                        .func_unique_id(&delete_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let small_lego_spec = small_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(delete_action_func)
        .func(small_lego_authoring_schema_func)
        .func(resource_payload_to_value_func)
        .schema(small_lego_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(small_lego_spec)?;
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
