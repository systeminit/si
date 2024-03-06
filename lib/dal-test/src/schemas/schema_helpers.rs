use dal::func::argument::FuncArgumentKind;
use dal::func::intrinsics::IntrinsicFunc;
use dal::BuiltinsError;
use si_pkg::{
    FuncArgumentSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData,
};

pub(crate) fn create_identity_func() -> FuncSpec {
    IntrinsicFunc::Identity
        .to_spec()
        .expect("create identity func spec")
}

pub(crate) async fn build_resource_payload_to_value_func() -> Result<FuncSpec, BuiltinsError> {
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

pub(crate) async fn build_asset_func(fn_name: &str) -> Result<FuncSpec, BuiltinsError> {
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
