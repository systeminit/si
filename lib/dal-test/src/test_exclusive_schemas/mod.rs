use category_pirate::migrate_test_exclusive_schema_pet_shop;
use category_pirate::migrate_test_exclusive_schema_pirate;
use category_validated::migrate_test_exclusive_schema_bad_validations;
use category_validated::migrate_test_exclusive_schema_validated_input;
use category_validated::migrate_test_exclusive_schema_validated_output;
use dal::builtins::schema;
use dal::func::argument::FuncArgumentKind;
use dal::func::intrinsics::IntrinsicFunc;
use dal::{BuiltinsError, BuiltinsResult, DalContext};
use dummy_secret::migrate_test_exclusive_schema_dummy_secret;
use fallout::migrate_test_exclusive_schema_fallout;
use katy_perry::migrate_test_exclusive_schema_katy_perry;
use legos::migrate_test_exclusive_schema_large_even_lego;
use legos::migrate_test_exclusive_schema_large_odd_lego;
use legos::migrate_test_exclusive_schema_medium_even_lego;
use legos::migrate_test_exclusive_schema_medium_odd_lego;
use legos::migrate_test_exclusive_schema_small_even_lego;
use legos::migrate_test_exclusive_schema_small_odd_lego;
use si_pkg::{
    FuncArgumentSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData,
};
use starfield::migrate_test_exclusive_schema_etoiles;
use starfield::migrate_test_exclusive_schema_morningstar;
use starfield::migrate_test_exclusive_schema_starfield;
use swifty::migrate_test_exclusive_schema_swifty;

mod category_pirate;
mod category_validated;
mod dummy_secret;
mod fallout;
mod katy_perry;
mod legos;
mod starfield;
mod swifty;

const PKG_VERSION: &str = "2019-06-03";
const PKG_CREATED_BY: &str = "System Initiative";
const SI_AWS_EC2_PKG: &str = "si-aws-ec2-2023-09-26.sipkg";
const SI_DOCKER_IMAGE_PKG: &str = "si-docker-image-2023-09-13.sipkg";
const SI_COREOS_PKG: &str = "si-coreos-2023-09-13.sipkg";

pub(crate) async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    schema::migrate_pkg(ctx, SI_DOCKER_IMAGE_PKG, None).await?;
    schema::migrate_pkg(ctx, SI_COREOS_PKG, None).await?;
    schema::migrate_pkg(ctx, SI_AWS_EC2_PKG, None).await?;
    migrate_test_exclusive_schema_starfield(ctx).await?;
    migrate_test_exclusive_schema_etoiles(ctx).await?;
    migrate_test_exclusive_schema_morningstar(ctx).await?;
    migrate_test_exclusive_schema_fallout(ctx).await?;
    migrate_test_exclusive_schema_dummy_secret(ctx).await?;
    migrate_test_exclusive_schema_swifty(ctx).await?;
    migrate_test_exclusive_schema_katy_perry(ctx).await?;
    migrate_test_exclusive_schema_pirate(ctx).await?;
    migrate_test_exclusive_schema_pet_shop(ctx).await?;
    migrate_test_exclusive_schema_validated_input(ctx).await?;
    migrate_test_exclusive_schema_validated_output(ctx).await?;
    migrate_test_exclusive_schema_bad_validations(ctx).await?;
    migrate_test_exclusive_schema_large_odd_lego(ctx).await?;
    migrate_test_exclusive_schema_large_even_lego(ctx).await?;
    migrate_test_exclusive_schema_medium_even_lego(ctx).await?;
    migrate_test_exclusive_schema_medium_odd_lego(ctx).await?;
    migrate_test_exclusive_schema_small_odd_lego(ctx).await?;
    migrate_test_exclusive_schema_small_even_lego(ctx).await?;
    Ok(())
}

fn create_identity_func() -> BuiltinsResult<FuncSpec> {
    Ok(IntrinsicFunc::Identity.to_spec()?)
}

fn build_resource_payload_to_value_func() -> BuiltinsResult<FuncSpec> {
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

fn build_action_func(code: &str, fn_name: &str) -> Result<FuncSpec, BuiltinsError> {
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

fn build_codegen_func(code: &str, fn_name: &str) -> BuiltinsResult<FuncSpec> {
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

fn build_asset_func(fn_name: &str) -> BuiltinsResult<FuncSpec> {
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
