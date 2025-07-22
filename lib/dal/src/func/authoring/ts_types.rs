//! This module contains types used for the [`Func`](crate::Func) authoring experience.

use crate::{
    FuncBackendKind,
    FuncBackendResponseType,
};

/*const SCHEMA_VARIANT_DEFINITION_TYPES: &str = concat!(
    include_str!("data/ts_types/asset_types_with_secrets.d.ts"),
    "\n",
    include_str!("data/ts_types/joi.d.ts"),
    "\n",
    "type Output = any;"
);*/

pub(crate) fn compile_return_types(
    response_type: FuncBackendResponseType,
    kind: FuncBackendKind,
) -> &'static str {
    if matches!(kind, FuncBackendKind::JsAttribute)
        && !matches!(
            response_type,
            FuncBackendResponseType::CodeGeneration | FuncBackendResponseType::Qualification
        )
    {
        return ""; // attribute functions have their output compiled dynamically
    }

    match response_type {
        FuncBackendResponseType::Boolean => "type Output = boolean | null;",
        FuncBackendResponseType::String => "type Output = string | null;",
        FuncBackendResponseType::Integer => "type Output = number | null;",
        FuncBackendResponseType::Qualification => {
            "type Output = {
  result: 'success' | 'warning' | 'failure';
  message?: string | null;
}"
        }
        FuncBackendResponseType::CodeGeneration => {
            "type Output = {
  format: string;
  code: string;
}"
        }
        FuncBackendResponseType::Validation => {
            "type Output = {
  valid: boolean;
  message: string;
}"
        }
        FuncBackendResponseType::Action => {
            "type Output = {
    resourceId?: string | null;
    status: 'ok' | 'warning' | 'error';
    payload?: { [key: string]: unknown } | null;
    message?: string | null;
}"
        }
        FuncBackendResponseType::Json => "type Output = any;",
        // Note: there is no ts function returning those
        FuncBackendResponseType::Identity => "interface Output extends Input {}",
        FuncBackendResponseType::Array => "type Output = any[];",
        FuncBackendResponseType::Map => "type Output = Record<string, any>;",
        FuncBackendResponseType::Object => "type Output = any;",
        FuncBackendResponseType::Unset => "type Output = undefined | null;",
        FuncBackendResponseType::Void => "type Output = void;",
        // All of the types for a management function are determined at "run"
        // time when compiling binding types
        FuncBackendResponseType::Management => "",
        _ => "",
        // we no longer serve this from the backend, its static on the front end
        //FuncBackendResponseType::SchemaVariantDefinition => SCHEMA_VARIANT_DEFINITION_TYPES
    }
}

// TODO: stop duplicating definition
// TODO: use execa types instead of any
// TODO: add os, fs and path types (possibly fetch but I think it comes with DOM)

// #[allow(missing_docs)]
pub(crate) fn compile_langjs_types() -> &'static str {
    "declare namespace YAML {
        function stringify(obj: unknown): string;
    }

    declare namespace template {
        function sourceOrValue(path: string, thisComponent: Input[\"thisComponent\"]): Output[\"ops\"][\"create\"][string][\"attributes\"][string];

        function checkUniqueNamePrefix(namePrefix: string, components: any): boolean;

        type ComponentWithSiNameAttributeValue = string | { '$source': any } | bool | number;
        interface ComponentWithSiName {
          attributes: {
            [path: string]: ComponentWithSiNameAttributeValue;
          }
        }

        export function getComponentName(component: ComponentScaffold): string;
    }

    declare namespace zlib {
        function gzip(inputstr: string, callback: any);
    }

    declare namespace requestStorage {
        function getEnv(key: string): string;
        function getItem(key: string): any;
        function getEnvKeys(): string[];
        function getKeys(): string[];
    }

    declare namespace extLib {
        function removeEmpty(object: any): any;
    }

    declare namespace siExec {

    interface WatchArgs {
        cmd: string,
        args?: readonly string[],
        execaOptions?: Options<string>,
        retryMs?: number,
        maxRetryCount?: number,
        callback: (child: execa.ExecaReturnValue<string>) => Promise<boolean>,
    }


    interface WatchResult {
        result: SiExecResult,
        failed?: 'deadlineExceeded' | 'commandFailed',
    }

    type SiExecResult = ExecaReturnValue<string>;

    async function waitUntilEnd(execaFile: string, execaArgs?: string[], execaOptions?: any): Promise<any>;
    async function watch(options: WatchArgs, deadlineCount?: number): Promise<WatchResult>;
}"
}
