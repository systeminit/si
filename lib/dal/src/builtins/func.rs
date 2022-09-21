use std::fs;
use std::fs::{read_dir, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::BuiltinsError::SerdeJson;
use crate::{
    BuiltinsError, BuiltinsResult, DalContext, Func, FuncBackendKind, FuncBackendResponseType,
    StandardModel,
};

#[derive(Deserialize, Serialize, Debug)]
struct FunctionMetadata {
    #[serde(skip)]
    name: String,
    kind: FuncBackendKind,
    response_type: FuncBackendResponseType,
    display_name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    code_file: Option<String>,
    code_entrypoint: Option<String>,
}

impl From<&Func> for FunctionMetadata {
    fn from(f: &Func) -> Self {
        let func_name_regex = Regex::new(r"si:(?P<name>.*)").unwrap();
        let func_name = func_name_regex
            .captures(f.name())
            .unwrap()
            .name("name")
            .unwrap()
            .as_str();

        let extension = match f.backend_kind() {
            FuncBackendKind::JsQualification
            | FuncBackendKind::JsCodeGeneration
            | FuncBackendKind::JsAttribute
            | FuncBackendKind::JsWorkflow
            | FuncBackendKind::JsCommand => Some("js"),

            _ => None,
        };

        let code_file = extension.map(|e| format!("{}.{}", func_name, e));

        FunctionMetadata {
            name: func_name.to_string(),
            kind: *f.backend_kind(),
            response_type: *f.backend_response_type(),
            // TODO Convert FunctionMetadata fields to use &str and remove these maps
            display_name: f.display_name().map(|s| s.to_string()),
            description: f.description().map(|s| s.to_string()),
            link: f.link().map(|s| s.to_string()),
            code_file,
            code_entrypoint: f.handler().map(|s| s.to_string()),
        }
    }
}

/// A private constant representing "/si/lib/dal".
const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    let extension_regex = Regex::new(r#"(?P<name>.*)\.(?P<extension>.*)"#)?;
    let mut path = PathBuf::from(CARGO_MANIFEST_DIR);
    path.push("src/builtins/func");
    let func_dir = read_dir(path.as_path())?;

    // JS functions
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

        let func_name = format!("si:{}", func_file_name);

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

        if let Some(code_file) = func_metadata.code_file {
            if func_metadata.code_entrypoint.is_none() {
                panic!("cannot create function with code_file but no code_entrypoint")
            }

            let mut func_path = path.clone();
            func_path.push(&code_file);

            let code = base64::encode(
                fs::read(func_path)
                    .unwrap_or_else(|err| panic!("Could not open '{}': {}", code_file, err)),
            );
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
    }

    // Rust functions
    migrate_rust_functions(ctx).await?;

    Ok(())
}

pub async fn persist(func: &Func) -> BuiltinsResult<()> {
    let new_metadata: FunctionMetadata = func.into();

    let mut base_path = PathBuf::from(CARGO_MANIFEST_DIR);
    base_path.push("src/builtins/func");

    if let Some(code_path) = new_metadata.code_file.as_ref() {
        let mut code_file_path = base_path.clone();
        code_file_path.push(code_path);

        let mut code_file = File::create(code_file_path)?;

        code_file.write_all(func.code_plaintext()?.unwrap().as_bytes())?;
    }

    let mut metadata_path = base_path.clone();
    metadata_path.push(format!("{}.json", new_metadata.name));
    let metadata_file = File::create(metadata_path)?;

    serde_json::to_writer_pretty(metadata_file, &new_metadata).map_err(SerdeJson)
}

/// Migrates [`Funcs`](crate::Func) that are Rust-based. These are likely temporary
/// as function execution should occur in an secure runtime environment.
async fn migrate_rust_functions(_ctx: &DalContext) -> BuiltinsResult<()> {
    // let _validate_string_array_func = Func::new(
    //     &ctx,
    //     "si:validateStringArray",
    //     FuncBackendKind::ValidateStringArrayValue,
    //     FuncBackendResponseType::Validation,
    // )
    // .await?;
    Ok(())
}
