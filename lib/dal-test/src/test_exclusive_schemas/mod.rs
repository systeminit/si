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
use std::str::FromStr;
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

const SCHEMA_ID_STARFIELD: &str = "01JARFRNN5VV1NMM0H5M63K3PY";
const SCHEMA_ID_PRIVATE_LANGUAGE: &str = "01JARFRNN5VV1NMM0H5M63K3PZ";
const SCHEMA_ID_ETOILES: &str = "01JARFRNN5VV1NMM0H5M63K3Q0";
const SCHEMA_ID_MORNINGSTAR: &str = "01JARFRNN5VV1NMM0H5M63K3Q1";
const SCHEMA_ID_FALLOUT: &str = "01JARFRNN5VV1NMM0H5M63K3Q2";
const SCHEMA_ID_DUMMY_SECRET: &str = "01JARFRNN5VV1NMM0H5M63K3Q3";
const SCHEMA_ID_SWIFTY: &str = "01JARFRNN5VV1NMM0H5M63K3Q4";
const SCHEMA_ID_KATY_PERRY: &str = "01JARFRNN5VV1NMM0H5M63K3Q5";
const SCHEMA_ID_PIRATE: &str = "01JARFRNN5VV1NMM0H5M63K3Q6";
const SCHEMA_ID_PET_SHOP: &str = "01JARFRNN5VV1NMM0H5M63K3Q7";
const SCHEMA_ID_VALIDATED_INPUT: &str = "01JARFRNN5VV1NMM0H5M63K3Q8";
const SCHEMA_ID_VALIDATED_OUTPUT: &str = "01JARFRNN5VV1NMM0H5M63K3Q9";
const SCHEMA_ID_BAD_VALIDATIONS: &str = "01JARFRNN5VV1NMM0H5M63K3QA";
const SCHEMA_ID_LARGE_ODD_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QB";
const SCHEMA_ID_LARGE_EVEN_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QC";
const SCHEMA_ID_MEDIUM_EVEN_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QD";
const SCHEMA_ID_MEDIUM_ODD_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QE";
const SCHEMA_ID_SMALL_ODD_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QF";
const SCHEMA_ID_SMALL_EVEN_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QG";
const SCHEMA_ID_FAKE_DOCKER_IMAGE: &str = "01JARFRNN5VV1NMM0H5M63K3QH";
const SCHEMA_ID_FAKE_BUTANE: &str = "01JARH2BTA5DK4J9Q4Q0XH46SR";

// allow expect here for the Ulid conversion. These will never panic.
#[allow(clippy::expect_used)]
pub(crate) async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    migrate_test_exclusive_func_si_resource_payload_to_value(ctx).await?;
    migrate_test_exclusive_schema_starfield(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_STARFIELD)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_private_language(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_PRIVATE_LANGUAGE)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_etoiles(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_ETOILES)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_morningstar(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_MORNINGSTAR)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_fallout(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_FALLOUT)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_dummy_secret(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_DUMMY_SECRET)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_swifty(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_SWIFTY)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_katy_perry(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_KATY_PERRY)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_pirate(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_PIRATE)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_pet_shop(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_PET_SHOP)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_validated_input(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_VALIDATED_INPUT)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_validated_output(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_VALIDATED_OUTPUT)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_bad_validations(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_BAD_VALIDATIONS)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_large_odd_lego(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_LARGE_ODD_LEGO)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_large_even_lego(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_LARGE_EVEN_LEGO)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_medium_even_lego(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_MEDIUM_EVEN_LEGO)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_medium_odd_lego(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_MEDIUM_ODD_LEGO)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_small_odd_lego(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_SMALL_ODD_LEGO)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_small_even_lego(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_SMALL_EVEN_LEGO)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_fake_docker_image(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_FAKE_DOCKER_IMAGE)
            .expect("should convert")
            .into(),
    )
    .await?;
    migrate_test_exclusive_schema_fake_butane(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_FAKE_BUTANE)
            .expect("should convert")
            .into(),
    )
    .await?;
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

fn build_management_func(code: &str, fn_name: &str) -> BuiltinsResult<FuncSpec> {
    Ok(FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(code)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::Management)
                .response_type(FuncSpecBackendResponseType::Management)
                .build()?,
        )
        .build()?)
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
