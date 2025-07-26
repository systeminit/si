use std::str::FromStr;

use category_pirate::{
    migrate_test_exclusive_schema_pet_shop,
    migrate_test_exclusive_schema_pirate,
};
use category_validated::{
    migrate_test_exclusive_schema_bad_validations,
    migrate_test_exclusive_schema_validated_input,
    migrate_test_exclusive_schema_validated_output,
};
use dal::{
    BuiltinsResult,
    DalContext,
    func::{
        argument::FuncArgumentKind,
        intrinsics::IntrinsicFunc,
    },
};
use dummy_double_secret::migrate_test_exclusive_schema_dummy_double_secret;
use dummy_secret::migrate_test_exclusive_schema_dummy_secret;
use fake_butane::migrate_test_exclusive_schema_fake_butane;
use fake_docker_image::migrate_test_exclusive_schema_fake_docker_image;
use fallout::migrate_test_exclusive_schema_fallout;
use katy_perry::migrate_test_exclusive_schema_katy_perry;
use legos::{
    migrate_test_exclusive_schema_large_even_lego,
    migrate_test_exclusive_schema_large_odd_lego,
    migrate_test_exclusive_schema_medium_even_lego,
    migrate_test_exclusive_schema_medium_odd_lego,
    migrate_test_exclusive_schema_small_even_lego,
    migrate_test_exclusive_schema_small_odd_lego,
};
use si_pkg::{
    FuncArgumentSpec,
    FuncSpec,
    FuncSpecBackendKind,
    FuncSpecBackendResponseType,
    FuncSpecData,
};
use starfield::{
    migrate_test_exclusive_schema_etoiles,
    migrate_test_exclusive_schema_morningstar,
    migrate_test_exclusive_schema_private_language,
    migrate_test_exclusive_schema_starfield,
};
use swifty::migrate_test_exclusive_schema_swifty;

mod category_pirate;
mod category_validated;
mod dummy_double_secret;
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

/// Schema id for the `dummy double secret` schema variant
pub const SCHEMA_ID_DUMMY_DOUBLE_SECRET: &str = "01JARFRNN5VV1NMM0H5M63K3QX";
/// Schema id for the `starfield` schema variant
pub const SCHEMA_ID_STARFIELD: &str = "01JARFRNN5VV1NMM0H5M63K3PY";
/// Schema id for the `private language` schema varitn
pub const SCHEMA_ID_PRIVATE_LANGUAGE: &str = "01JARFRNN5VV1NMM0H5M63K3PZ";
/// Schema id for the `etoiles` schema variant
pub const SCHEMA_ID_ETOILES: &str = "01JARFRNN5VV1NMM0H5M63K3Q0";
/// Schema id for the `morningstar` schema variant
pub const SCHEMA_ID_MORNINGSTAR: &str = "01JARFRNN5VV1NMM0H5M63K3Q1";
/// Schema id for the `fallout` schema variant
pub const SCHEMA_ID_FALLOUT: &str = "01JARFRNN5VV1NMM0H5M63K3Q2";
/// Schema id for the `dummy secret` schema variant
pub const SCHEMA_ID_DUMMY_SECRET: &str = "01JARFRNN5VV1NMM0H5M63K3Q3";
/// Schema id for the `swifty` schema variant
pub const SCHEMA_ID_SWIFTY: &str = "01JARFRNN5VV1NMM0H5M63K3Q4";
/// Schema id for the `katy perry` schema variant
pub const SCHEMA_ID_KATY_PERRY: &str = "01JARFRNN5VV1NMM0H5M63K3Q5";
/// Schema id for the `pirate` schema variant
pub const SCHEMA_ID_PIRATE: &str = "01JARFRNN5VV1NMM0H5M63K3Q6";
/// Schema id for the `pet shop` schema variant
pub const SCHEMA_ID_PET_SHOP: &str = "01JARFRNN5VV1NMM0H5M63K3Q7";
/// Schema id for the `validated input` schema variant
pub const SCHEMA_ID_VALIDATED_INPUT: &str = "01JARFRNN5VV1NMM0H5M63K3Q8";
/// Schema id for the `validated output` schema variant
pub const SCHEMA_ID_VALIDATED_OUTPUT: &str = "01JARFRNN5VV1NMM0H5M63K3Q9";
/// Schema id for the `bad validations` schema variant
pub const SCHEMA_ID_BAD_VALIDATIONS: &str = "01JARFRNN5VV1NMM0H5M63K3QA";
/// Schema id for the `large odd lego` schema variant
pub const SCHEMA_ID_LARGE_ODD_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QB";
/// Schema id for the `large even lego` schema variant
pub const SCHEMA_ID_LARGE_EVEN_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QC";
/// Schema id for the `medium odd lego` schema variant
pub const SCHEMA_ID_MEDIUM_EVEN_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QD";
/// Schema id for the `medium even lego` schema variant
pub const SCHEMA_ID_MEDIUM_ODD_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QE";
/// Schema id for the `small odd lego` schema variant
pub const SCHEMA_ID_SMALL_ODD_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QF";
/// Schema id for the `small even lego` schema variant
pub const SCHEMA_ID_SMALL_EVEN_LEGO: &str = "01JARFRNN5VV1NMM0H5M63K3QG";
/// Schema id for the `fake docker image` schema variant
pub const SCHEMA_ID_FAKE_DOCKER_IMAGE: &str = "01JARFRNN5VV1NMM0H5M63K3QH";
/// Schema id for the `fake butane` schema variant
pub const SCHEMA_ID_FAKE_BUTANE: &str = "01JARH2BTA5DK4J9Q4Q0XH46SR";

/// Install the test schemas
/// allow expect here for the Ulid conversion. These will never panic.
#[allow(clippy::expect_used)]
pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
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
    migrate_test_exclusive_schema_dummy_double_secret(
        ctx,
        ulid::Ulid::from_str(SCHEMA_ID_DUMMY_DOUBLE_SECRET)
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

fn create_identity_func() -> BuiltinsResult<FuncSpec> {
    Ok(IntrinsicFunc::Identity.to_spec()?)
}

fn build_management_func(code: &str, fn_name: &str) -> BuiltinsResult<FuncSpec> {
    Ok(FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(code)
                .display_name(fn_name.to_string())
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
