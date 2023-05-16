use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    func::argument::{FuncArgument, FuncArgumentKind},
    ActionKind, ActionPrototype, ActionPrototypeContext, BuiltinsError, BuiltinsResult, DalContext,
    Func, FuncBackendKind, FuncBackendResponseType, Schema, StandardModel,
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

// This is a transitional function that creates the necessary action prototypes during
// migration so that we have the correct data for the initial SI package.
pub async fn migrate_action_prototypes(ctx: &DalContext) -> BuiltinsResult<()> {
    let command_functions = vec![
        "si:awsAmiRefreshAction",
        "si:awsEc2CreateAction",
        "si:awsEc2DeleteAction",
        "si:awsEc2RefreshAction",
        "si:awsEgressCreateAction",
        "si:awsEgressRefreshAction",
        "si:awsEgressDeleteAction",
        "si:awsIngressCreateAction",
        "si:awsIngressRefreshAction",
        "si:awsIngressDeleteAction",
        "si:awsEipCreateAction",
        "si:awsEipDeleteAction",
        "si:awsEipRefreshAction",
        "si:awsKeyPairCreateAction",
        "si:awsKeyPairDeleteAction",
        "si:awsKeyPairRefreshAction",
        "si:awsSecurityGroupCreateAction",
        "si:awsSecurityGroupDeleteAction",
        "si:awsSecurityGroupRefreshAction",
        "si:awsRegionRefreshAction",
        "si:dockerImageRefreshAction",
    ];

    for func_name in command_functions {
        let schema_name = match func_name {
            "si:awsEc2DeleteAction" | "si:awsEc2CreateAction" | "si:awsEc2RefreshAction" => {
                "EC2 Instance"
            }
            "si:awsSecurityGroupRefreshAction"
            | "si:awsSecurityGroupCreateAction"
            | "si:awsSecurityGroupDeleteAction" => "Security Group",
            "si:awsEgressCreateAction"
            | "si:awsEgressRefreshAction"
            | "si:awsEgressDeleteAction" => "Egress",
            "si:awsEipCreateAction" | "si:awsEipDeleteAction" | "si:awsEipRefreshAction" => {
                "Elastic IP"
            }
            "si:awsAmiRefreshAction" => "AMI",
            "si:awsIngressDeleteAction"
            | "si:awsIngressRefreshAction"
            | "si:awsIngressCreateAction" => "Ingress",
            "si:awsRegionRefreshAction" => "Region",
            "si:awsKeyPairCreateAction"
            | "si:awsKeyPairRefreshAction"
            | "si:awsKeyPairDeleteAction" => "Key Pair",
            "si:dockerImageRefreshAction" => "Docker Image",
            _ => unreachable!("that string is not in my list!"),
        };

        let action_kind = match func_name {
            "si:awsAmiRefreshAction"
            | "si:awsEc2RefreshAction"
            | "si:awsEgressRefreshAction"
            | "si:awsEipRefreshAction"
            | "si:awsIngressRefreshAction"
            | "si:awsRegionRefreshAction"
            | "si:awsSecurityGroupRefreshAction"
            | "si:dockerImageRefreshAction"
            | "si:awsKeyPairRefreshAction" => ActionKind::Refresh,
            "si:awsEc2CreateAction"
            | "si:awsEgressCreateAction"
            | "si:awsEipCreateAction"
            | "si:awsIngressCreateAction"
            | "si:awsKeyPairCreateAction"
            | "si:awsSecurityGroupCreateAction" => ActionKind::Create,
            "si:awsEc2DeleteAction"
            | "si:awsEgressDeleteAction"
            | "si:awsEipDeleteAction"
            | "si:awsIngressDeleteAction"
            | "si:awsKeyPairDeleteAction"
            | "si:awsSecurityGroupDeleteAction" => ActionKind::Delete,
            _ => unreachable!("that string is not in my list :("),
        };

        let schema = Schema::find_by_attr(ctx, "name", &schema_name)
            .await?
            .pop()
            .expect("able to find default schema");

        let default_variant = schema.default_variant(ctx).await?;

        let func = Func::find_by_name(ctx, func_name)
            .await?
            .expect("cannot find builtin command function");

        let context = ActionPrototypeContext {
            schema_variant_id: *default_variant.id(),
        };

        if ActionPrototype::find_for_func_kind_context(ctx, *func.id(), action_kind, context)
            .await
            .expect("able to search for the action prototype")
            .is_empty()
        {
            info!("migrating action prototype for {}", func.name());

            ActionPrototype::new(ctx, *func.id(), action_kind, context)
                .await
                .expect("could not create action prototype");
        }
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
