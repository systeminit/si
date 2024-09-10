use base64::engine::general_purpose;
use base64::Engine;
use category_pirate::migrate_test_exclusive_schema_pet_shop;
use category_pirate::migrate_test_exclusive_schema_pirate;
use category_validated::migrate_test_exclusive_schema_bad_validations;
use category_validated::migrate_test_exclusive_schema_validated_input;
use category_validated::migrate_test_exclusive_schema_validated_output;
use dal::func::argument::FuncArgument;
use dal::func::argument::FuncArgumentKind;
use dal::func::intrinsics::IntrinsicFunc;
use dal::Func;
use dal::FuncBackendKind;
use dal::FuncBackendResponseType;
use dal::{BuiltinsResult, DalContext};
use dummy_secret::migrate_test_exclusive_schema_dummy_secret;
use fake_butane::migrate_test_exclusive_schema_fake_butane;
use fake_docker_image::migrate_test_exclusive_schema_fake_docker_image;
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
use starfield::migrate_test_exclusive_schema_private_language;
use starfield::migrate_test_exclusive_schema_starfield;
use swifty::migrate_test_exclusive_schema_swifty;

mod category_pirate;
mod category_validated;
mod dummy_secret;
mod fake_butane;
mod fake_docker_image;
mod fallout;
mod katy_perry;
mod legos;
mod starfield;
mod swifty;

const PKG_VERSION: &str = "2019-06-03";
const PKG_CREATED_BY: &str = "System Initiative";

pub(crate) async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    migrate_test_exclusive_func_si_resource_payload_to_value(ctx).await?;
    migrate_test_exclusive_schema_starfield(ctx).await?;
    migrate_test_exclusive_schema_private_language(ctx).await?;
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
    migrate_test_exclusive_schema_fake_docker_image(ctx).await?;
    migrate_test_exclusive_schema_fake_butane(ctx).await?;
    Ok(())
}

// TODO(nick): remove this if "si:resourcePayloadToValue" becomes an instrinsic func.
async fn migrate_test_exclusive_func_si_resource_payload_to_value(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let func_name = "si:resourcePayloadToValue";

    let func_id = match Func::find_id_by_name(ctx, func_name).await? {
        Some(existing_func_id) => existing_func_id,
        None => {
            let new_func = Func::new(
                ctx,
                func_name,
                None::<String>,
                None::<String>,
                None::<String>,
                false,
                true,
                FuncBackendKind::JsAttribute,
                FuncBackendResponseType::Json,
                Some("main"),
                Some(general_purpose::STANDARD_NO_PAD.encode("async function main(arg: Input): Promise<Output> { return arg.payload ?? {}; }")),
            )
            .await?;
            new_func.id
        }
    };

    if FuncArgument::find_by_name_for_func(ctx, "payload", func_id)
        .await?
        .is_none()
    {
        FuncArgument::new(ctx, "payload", FuncArgumentKind::Object, None, func_id).await?;
    }

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

fn build_action_func(code: &str, fn_name: &str) -> BuiltinsResult<FuncSpec> {
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
