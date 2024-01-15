use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use telemetry::prelude::*;

use crate::{
    func::{
        argument::{FuncArgument, FuncArgumentKind},
        intrinsics::IntrinsicFunc,
    },
    installed_pkg::InstalledPkg,
    pkg::import_pkg_from_pkg,
    BuiltinsError, BuiltinsResult, DalContext, Func, FuncBackendKind, FuncBackendResponseType,
    StandardModel,
};

#[derive(Deserialize, Serialize, Debug)]
struct FunctionMetadataArgument {
    name: String,
    kind: FuncArgumentKind,
}

#[derive(Deserialize, Serialize, Debug)]
struct FunctionMetadata {
    kind: FuncBackendKind,
    arguments: Option<Vec<FunctionMetadataArgument>>,
    response_type: FuncBackendResponseType,
    hidden: Option<bool>,
    display_name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    code_file: Option<String>,
    code_entrypoint: Option<String>,
}

/// We want the src/builtins/func/** files to be available at run time inside of the Docker container
/// that we build, but it would be nice to not have to include arbitrary bits of the source tree when
/// building it. This allows us to compile the builtins into the binary, so they're already available
/// in memory.
///
/// The instances of this end up in a magic `ASSETS` array const.
#[iftree::include_file_tree("paths = '/src/builtins/func/**'")]
pub struct FuncBuiltin {
    relative_path: &'static str,
    contents_str: &'static str,
}

static FUNC_BUILTIN_BY_PATH: once_cell::sync::Lazy<std::collections::HashMap<&str, &FuncBuiltin>> =
    once_cell::sync::Lazy::new(|| {
        ASSETS
            .iter()
            .map(|func_builtin| (func_builtin.relative_path, func_builtin))
            .collect()
    });

pub async fn migrate_intrinsics(ctx: &DalContext) -> BuiltinsResult<()> {
    let intrinsics_pkg_spec = IntrinsicFunc::pkg_spec()?;
    let _name = intrinsics_pkg_spec.name.to_owned();
    let intrinsics_pkg = SiPkg::load_from_spec(intrinsics_pkg_spec)?;

    if InstalledPkg::find_by_hash(ctx, &intrinsics_pkg.hash()?.to_string())
        .await?
        .is_none()
    {
        import_pkg_from_pkg(ctx, &intrinsics_pkg, None, true).await?;
        ctx.blocking_commit().await?;
    }

    Ok(())
}

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    for builtin_func_file in ASSETS.iter() {
        let builtin_path = std::path::Path::new(builtin_func_file.relative_path);
        match builtin_path.extension() {
            Some(extension) => {
                if extension != std::ffi::OsStr::new("json") {
                    debug!("skipping {:?}: not a json file", builtin_path);
                    continue;
                }
            }
            None => {
                warn!("skipping {:?}: no file extension", builtin_path);
                continue;
            }
        };

        let func_metadata: FunctionMetadata = serde_json::from_str(builtin_func_file.contents_str)
            .map_err(|e| BuiltinsError::FuncJson(builtin_path.to_string_lossy().to_string(), e))?;

        let func_name = format!(
            "si:{}",
            builtin_path
                .file_stem()
                .ok_or_else(|| {
                    BuiltinsError::FuncMetadata(format!(
                        "Unable to determine base file name for {builtin_path:?}"
                    ))
                })?
                .to_string_lossy()
        );

        let mut existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
        if let Some(mut existing_func) = existing_func.pop() {
            if *existing_func.backend_kind() != func_metadata.kind {
                info!(
                    "updating backend kind for {:?} from {:?} to {:?}",
                    &func_name,
                    *existing_func.backend_kind(),
                    func_metadata.kind
                );
                existing_func
                    .set_backend_kind(ctx, func_metadata.kind)
                    .await?;
            }

            warn!("skipping {:?}: func already exists", &func_name);
            continue;
        }

        let mut new_func = Func::new(
            ctx,
            &func_name,
            func_metadata.kind,
            func_metadata.response_type,
        )
        .await
        .expect("cannot create func");

        if let Some(code_file) = func_metadata.code_file {
            if func_metadata.code_entrypoint.is_none() {
                panic!("cannot create function with code_file but no code_entrypoint")
            }

            let metadata_base_path = builtin_path.parent().ok_or_else(|| {
                BuiltinsError::FuncMetadata(format!(
                    "Cannot determine parent path of {builtin_path:?}"
                ))
            })?;
            let func_path = metadata_base_path.join(std::path::Path::new(&code_file));

            let code = FUNC_BUILTIN_BY_PATH
                .get(func_path.as_os_str().to_str().ok_or_else(|| {
                    BuiltinsError::FuncMetadata(format!("Unable to convert {func_path:?} to &str"))
                })?)
                .ok_or_else(|| {
                    BuiltinsError::FuncMetadata(format!("Code file not found: {code_file:?}"))
                })?;
            let code = general_purpose::STANDARD_NO_PAD.encode(code.contents_str);
            new_func
                .set_code_base64(ctx, Some(code))
                .await
                .expect("cannot set code");
        }

        new_func
            .set_handler(ctx, func_metadata.code_entrypoint)
            .await
            .expect("cannot set handler");

        new_func
            .set_display_name(ctx, func_metadata.display_name)
            .await
            .expect("cannot set display name");
        new_func
            .set_description(ctx, func_metadata.description)
            .await
            .expect("cannot set func description");
        new_func
            .set_link(ctx, func_metadata.link)
            .await
            .expect("cannot set func link");
        new_func
            .set_hidden(ctx, func_metadata.hidden.unwrap_or(false))
            .await
            .expect("cannot set func hidden");
        new_func
            .set_builtin(ctx, true)
            .await
            .expect("cannot set func builtin");

        if let Some(arguments) = func_metadata.arguments {
            for arg in arguments {
                FuncArgument::new(ctx, &arg.name, arg.kind, None, *new_func.id()).await?;
            }
        }
        ctx.blocking_commit().await?;
    }

    Ok(())
}
