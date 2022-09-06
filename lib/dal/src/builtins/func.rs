use std::fs;
use std::fs::{read_dir, File};
use std::io::BufReader;
use std::path::PathBuf;

use regex::Regex;
use serde::Deserialize;

use crate::{
    BuiltinsError, BuiltinsResult, DalContext, Func, FuncBackendKind, FuncBackendResponseType,
    StandardModel,
};

#[derive(Deserialize, Debug)]
struct FunctionMetadata {
    name: Option<String>,
    kind: FuncBackendKind,
    response_type: FuncBackendResponseType,
    display_name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    code: Option<String>,
    code_file: Option<String>,
    code_entrypoint: Option<String>,
}

/// A private constant representing "/si/lib/dal".
const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub async fn migrate(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
    let extension_regex = Regex::new(r#"(?P<name>.*)\.(?P<extension>.*)"#)?;
    let mut path = PathBuf::from(CARGO_MANIFEST_DIR);
    path.push("src/builtins/func");
    let func_dir = read_dir(path.as_path())?;

    for entry_result in func_dir {
        let entry = entry_result?;

        let file_name = match entry.file_name().into_string() {
            Ok(file_name) => file_name,
            Err(file_name_error) => {
                panic!(
                    "Could not convert {} to String",
                    file_name_error.to_str().unwrap()
                );
            }
        };

        let captures = extension_regex.captures(file_name.as_str()).unwrap();

        let func_file_name = captures.name("name").unwrap().as_str();
        let func_file_extension = captures.name("extension").unwrap().as_str();

        if func_file_extension != "json" {
            continue;
        }

        let func_metadata: FunctionMetadata = {
            let file = File::open(entry.path())?;
            serde_json::from_reader(BufReader::new(file))
                .map_err(|e| BuiltinsError::SerdeJsonErrorForFunc(func_file_name.to_string(), e))?
        };

        let func_name = format!(
            "si:{}",
            match &func_metadata.name {
                None => func_file_name,
                Some(name) => name,
            }
        );

        let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;

        if !existing_func.is_empty() {
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

        if func_metadata.code_file.is_some() && func_metadata.code.is_some() {
            panic!("cannot create function with both code_file and code")
        }

        if let Some(code_file) = func_metadata.code_file {
            if func_metadata.code_entrypoint.is_none() {
                panic!("cannot create function with code_file but no code_entrypoint")
            }

            let mut func_path = path.clone();
            func_path.push(code_file);

            let code = base64::encode(fs::read(func_path)?);
            new_func
                .set_code_base64(ctx, Some(code))
                .await
                .expect("cannot set code");
        }

        if let Some(code) = func_metadata.code {
            if func_metadata.code_entrypoint.is_none() {
                panic!("cannot create function with code but no code_entrypoint")
            }

            let code = base64::encode(code);
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
            .expect("Set func description");
        new_func
            .set_link(ctx, func_metadata.link)
            .await
            .expect("set func link");
    }

    Ok(())
}
