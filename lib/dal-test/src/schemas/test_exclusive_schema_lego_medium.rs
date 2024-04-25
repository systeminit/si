use crate::schemas::schema_helpers::{
    build_action_func, build_asset_func, build_resource_payload_to_value_func, create_identity_func,
};
use dal::pkg::import_pkg_from_pkg;
use dal::{prop::PropPath, ComponentType, DeprecatedActionKind};
use dal::{BuiltinsResult, DalContext, PropKind};
use si_pkg::SchemaSpecData;
use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, PkgSpec, PropSpec, SchemaSpec,
    SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpec, SocketSpecData, SocketSpecKind,
};

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
                        .name("one")
                        .kind(PropKind::String)
                        .default_value(serde_json::json!("0"))
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("one")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("two")
                        .kind(PropKind::String)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("three")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("three")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("four")
                        .kind(PropKind::String)
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("one")
                        .data(
                            SocketSpecData::builder()
                                .name("one")
                                .connection_annotations(serde_json::to_string(&vec!["one"])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("three")
                        .data(
                            SocketSpecData::builder()
                                .name("three")
                                .connection_annotations(serde_json::to_string(&vec!["three"])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("two")
                        .data(
                            SocketSpecData::builder()
                                .name("two")
                                .connection_annotations(serde_json::to_string(&vec!["two"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "two"]))
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("four")
                        .data(
                            SocketSpecData::builder()
                                .name("four")
                                .connection_annotations(serde_json::to_string(&vec!["four"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "four"]))
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&DeprecatedActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&DeprecatedActionKind::Refresh)
                        .func_unique_id(&refresh_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&DeprecatedActionKind::Other)
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
                        .name("two")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("two")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("one")
                        .kind(PropKind::String)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("four")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("four")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("three")
                        .kind(PropKind::String)
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("two")
                        .data(
                            SocketSpecData::builder()
                                .name("two")
                                .connection_annotations(serde_json::to_string(&vec!["two"])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("four")
                        .data(
                            SocketSpecData::builder()
                                .name("four")
                                .connection_annotations(serde_json::to_string(&vec!["four"])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("one")
                        .data(
                            SocketSpecData::builder()
                                .name("one")
                                .connection_annotations(serde_json::to_string(&vec!["one"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "one"]))
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("three")
                        .data(
                            SocketSpecData::builder()
                                .name("three")
                                .connection_annotations(serde_json::to_string(&vec!["three"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "three"]))
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&DeprecatedActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&DeprecatedActionKind::Refresh)
                        .func_unique_id(&refresh_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&DeprecatedActionKind::Other)
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
