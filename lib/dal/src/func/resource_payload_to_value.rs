use crate::{pkg::import_pkg_from_pkg, DalContext};
use chrono::DateTime;
use si_pkg::{
    FuncArgumentKind, FuncArgumentSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType,
    FuncSpecData, PkgSpec, SiPkg,
};

use crate::func::{FuncError, FuncResult};

use super::Func;

pub fn build_resource_payload_to_value_pkg() -> FuncResult<PkgSpec> {
    let payload_to_val = "async function main(arg: Input): Promise<Output> {
  return arg.payload ?? {};
}";
    let fn_name = "si:resourcePayloadToValue";
    let payload_to_val = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(payload_to_val)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Object)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("payload")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .build()?;

    let mut builder = PkgSpec::builder();
    builder.name("si-resource-payload-to-value");
    builder.version("2024-11-22");
    builder.created_at(DateTime::parse_from_rfc2822(
        "Fri, 22 Nov 2024 00:00:00 EDT",
    )?);
    builder.created_by("System Initiative");
    builder.func(payload_to_val);

    builder
        .build()
        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))
}

pub async fn install_resource_payload_to_value_if_missing(ctx: &DalContext) -> FuncResult<()> {
    if Func::find_id_by_name(ctx, "si:resourcePayloadToValue")
        .await?
        .is_none()
    {
        let spec = build_resource_payload_to_value_pkg()?;
        import_pkg_from_pkg(ctx, &SiPkg::load_from_spec(spec)?, None)
            .await
            .map_err(Box::new)?;
    }
    Ok(())
}
