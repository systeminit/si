use dal::func::argument::FuncArgumentKind;
use dal::func::intrinsics::IntrinsicFunc;
use dal::{BuiltinsError, BuiltinsResult};
use si_pkg::{
    FuncArgumentSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData,
};

pub(crate) fn create_identity_func() -> BuiltinsResult<FuncSpec> {
    Ok(IntrinsicFunc::Identity.to_spec()?)
}

pub(crate) async fn build_resource_payload_to_value_func() -> BuiltinsResult<FuncSpec> {
    let resource_payload_to_value_func_code = "async function main(arg: Input): Promise<Output> {\
            return arg.payload ?? {};
        }";
    let fn_name = "test:resourcePayloadToValue";
    let resource_payload_to_value_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(resource_payload_to_value_func_code)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Json)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("payload")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .build()?;

    Ok(resource_payload_to_value_func)
}

pub(crate) async fn build_action_func(
    code: &str,
    fn_name: &str,
) -> Result<FuncSpec, BuiltinsError> {
    let func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(code)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::JsAction)
                .response_type(FuncSpecBackendResponseType::Action)
                .build()?,
        )
        .build()?;

    Ok(func)
}

pub(crate) async fn build_codegen_func(code: &str, fn_name: &str) -> BuiltinsResult<FuncSpec> {
    let func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .argument(
            FuncArgumentSpec::builder()
                .name("domain")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(code)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::CodeGeneration)
                .build()?,
        )
        .build()?;

    Ok(func)
}

pub(crate) async fn build_asset_func(fn_name: &str) -> BuiltinsResult<FuncSpec> {
    let scaffold_func = "function main() {\
                return new AssetBuilder().build();
            }";
    let asset_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(scaffold_func)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    Ok(asset_func)
}
