use dal::{
    BuiltinsResult,
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

use crate::{
    helpers::secret::assemble_dummy_secret_socket_and_prop,
    test_exclusive_schemas::{
        PKG_CREATED_BY,
        PKG_VERSION,
        build_action_func,
        create_identity_func,
    },
};

pub(crate) async fn migrate_test_exclusive_schema_fallout(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut fallout_builder = PkgSpec::builder();

    let schema_name = "fallout";

    fallout_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    let (fallout_create_action_func, fallout_delete_action_func) = action_funcs()?;

    let fallout_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldFalloutAsset";
    let fallout_authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fallout_scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let secret_definition_name = "dummy";

    let (dummy_secret_input_scoket, dummy_secret_prop) =
        assemble_dummy_secret_socket_and_prop(&identity_func_spec, secret_definition_name)?;

    let fallout_schema = SchemaSpec::builder()
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
                .unique_id("fallout_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&fallout_authoring_schema_func.unique_id)
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
                .domain_prop(
                    PropSpec::builder()
                        .name("special")
                        .kind(PropKind::String)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("rads")
                        .kind(PropKind::Integer)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("active")
                        .kind(PropKind::Boolean)
                        .default_value(serde_json::json!(true))
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("bethesda")
                        .data(
                            SocketSpecData::builder()
                                .name("bethesda")
                                .connection_annotations(serde_json::to_string(&vec!["bethesda"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "special"]))
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
                                .connection_annotations(serde_json::to_string(&vec![
                                    "fallout", "bethesda",
                                ])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root"]))
                                .build()?,
                        )
                        .build()?,
                )
                .secret_prop(dummy_secret_prop)
                .socket(dummy_secret_input_scoket)
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Create)
                        .func_unique_id(&fallout_create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Destroy)
                        .func_unique_id(&fallout_delete_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let fallout_spec = fallout_builder
        .func(identity_func_spec)
        .func(fallout_create_action_func)
        .func(fallout_delete_action_func)
        .func(fallout_authoring_schema_func)
        .schema(fallout_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(fallout_spec)?;
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

fn action_funcs() -> BuiltinsResult<(FuncSpec, FuncSpec)> {
    // Add the action create func.
    let code = "async function main() {
        const authCheck = requestStorage.getItem('dummySecretString');
        const workspaceToken = requestStorage.getItem('workspaceToken');
        if (authCheck && workspaceToken) {
            if (authCheck === 'todd' && workspaceToken === 'token') {
                return {
                    status: 'ok',
                    payload: {
                        'poop': true
                    }
                };
            }
            return {
                status: 'error',
                message: 'cannot create: dummy secret string does not match expected value'
            };
        } else {
            return {
                status: 'error',
                message: 'cannot create: dummy secret string is empty'
            };
        }
    }";
    let fn_name = "test:createActionFallout";
    let fallout_create_action_func = build_action_func(code, fn_name)?;

    // Add the action delete func.
    let delete_action_code = "async function main() {
        return {
            status: 'ok',
            payload: undefined
        };
    }";
    let fn_name = "test:deleteActionFallout";
    let fallout_delete_action_func = build_action_func(delete_action_code, fn_name)?;

    Ok((fallout_create_action_func, fallout_delete_action_func))
}
