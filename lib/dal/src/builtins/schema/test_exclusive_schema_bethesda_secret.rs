use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, AuthenticationFuncSpec, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, LeafFunctionSpec, LeafInputLocation,
    LeafKind, PkgSpec, PropSpec, PropSpecKind, PropSpecWidgetKind, SchemaSpec, SchemaVariantSpec,
    SchemaVariantSpecData, SiPkg, SocketSpec, SocketSpecArity, SocketSpecData, SocketSpecKind,
};
use si_pkg::{FuncSpecData, SchemaSpecData};

use crate::func::argument::FuncArgumentKind;
use crate::func::intrinsics::IntrinsicFunc;
use crate::pkg::import_pkg_from_pkg;
use crate::prop::PropPath;
use crate::{BuiltinsResult, DalContext};

pub async fn migrate_test_exclusive_schema_bethesda_secret(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "bethesda-secret";

    let mut builder = PkgSpec::builder();

    builder
        .name(name)
        .version("2024-03-01")
        .created_by("System Initiative");

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let scaffold_func = "function createAsset() {\
        return new AssetBuilder().build()
    }";
    let fn_name = "test:scaffoldBethesdaSecretAsset";
    let authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let auth_func_code = "async function auth(secret: Input): Promise<Output> { requestStorage.setItem('fakeSecretString', secret.value); }";
    let fn_name = "test:setFakeSecretString";
    let auth_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(auth_func_code)
                .handler("auth")
                .backend_kind(FuncSpecBackendKind::JsAuthentication)
                .response_type(FuncSpecBackendResponseType::Json)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("secret")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .build()?;

    let (fake_secret_definition_prop, fake_secret_prop, fake_secret_output_socket) =
        assemble_secret_definition_fake(&identity_func_spec)?;

    let (
        qualification_fake_secret_value_is_todd_func,
        qualification_fake_secret_value_is_todd_leaf,
    ) = assemble_qualification_fake_secret_value_is_todd()?;

    let schema = SchemaSpec::builder()
        .name(name)
        .data(
            SchemaSpecData::builder()
                .name(name)
                .category("test exclusive")
                .category_name(name)
                .build()
                .expect("build schema spec data"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id(format!("{name}_sv"))
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&authoring_schema_func.unique_id)
                        .build()?,
                )
                .auth_func(
                    AuthenticationFuncSpec::builder()
                        .func_unique_id(&auth_func.unique_id)
                        .build()?,
                )
                .secret_prop(fake_secret_prop)
                .secret_definition_prop(fake_secret_definition_prop)
                .socket(fake_secret_output_socket)
                .leaf_function(qualification_fake_secret_value_is_todd_leaf)
                .build()?,
        )
        .build()?;

    let spec = builder
        .func(identity_func_spec)
        .func(authoring_schema_func)
        .func(auth_func)
        .func(qualification_fake_secret_value_is_todd_func)
        .schema(schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(spec)?;
    // TODO(nick): decide what to do with override schema builtin featuee flag.
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(crate::pkg::ImportOptions {
            schemas: Some(vec![name.into()]),
            ..Default::default()
        }),
        true,
    )
    .await?;

    Ok(())
}

// Mimics the "defineSecret" function in "asset_builder.ts".
fn assemble_secret_definition_fake(
    identity_func_spec: &FuncSpec,
) -> BuiltinsResult<(PropSpec, PropSpec, SocketSpec)> {
    let secret_definition_name = "fake";

    // First, create the child of "/root/secret_definition" that defines our secret.
    let new_secret_definition_prop = PropSpec::builder()
        .name("value")
        .kind(PropSpecKind::String)
        .widget_kind(PropSpecWidgetKind::Password)
        .build()?;

    // Second, add it as a new prop underneath "/root/secrets" object. Make sure the "secretKind" is available.
    let new_secret_prop = PropSpec::builder()
        .name(secret_definition_name)
        .kind(PropSpecKind::String)
        .widget_kind(PropSpecWidgetKind::Secret)
        .widget_options(serde_json::json![
            [
                {
                    "label": "secretKind",
                    "value": secret_definition_name
                }
            ]
        ])
        .build()?;

    // Third, add an output socket for other components to use the secret.
    let new_secret_output_socket = SocketSpec::builder()
        .name(secret_definition_name)
        .data(
            SocketSpecData::builder()
                .name(secret_definition_name)
                .connection_annotations(serde_json::to_string(&vec![
                    secret_definition_name.to_lowercase()
                ])?)
                .kind(SocketSpecKind::Output)
                .arity(SocketSpecArity::One)
                .func_unique_id(&identity_func_spec.unique_id)
                .build()?,
        )
        .input(
            AttrFuncInputSpec::builder()
                .name("identity")
                .kind(AttrFuncInputSpecKind::Prop)
                .prop_path(PropPath::new(["root", "secrets", secret_definition_name]))
                .build()?,
        )
        .build()?;

    Ok((
        new_secret_definition_prop,
        new_secret_prop,
        new_secret_output_socket,
    ))
}

fn assemble_qualification_fake_secret_value_is_todd() -> BuiltinsResult<(FuncSpec, LeafFunctionSpec)>
{
    let fn_code = "async function qualification(_component: Input): Promise<Output> {\
        const authCheck = requestStorage.getItem('fakeSecretString');
        if (authCheck) {
            if (authCheck === 'todd') {
                return {
                    result: 'success',
                    message: 'fake secret string matches expected value'
                };
            }
            return {
                result: 'failure',
                message: 'fake secret string does not match expected value'
            };
        } else {
            return {
                result: 'failure',
                message: 'fake secret string is empty'
            };
        }
    }";
    let fn_name = "test:qualificationFakeSecretStringIsTodd";
    let qualification_fake_secret_value_is_todd_func = FuncSpec::builder()
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

    let qualification_fake_secret_value_is_todd_leaf = LeafFunctionSpec::builder()
        .func_unique_id(&qualification_fake_secret_value_is_todd_func.unique_id)
        .leaf_kind(LeafKind::Qualification)
        .inputs(vec![LeafInputLocation::Secrets])
        .build()?;

    Ok((
        qualification_fake_secret_value_is_todd_func,
        qualification_fake_secret_value_is_todd_leaf,
    ))
}
