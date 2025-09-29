import { AuthenticationFuncSpec } from "../bindings/AuthenticationFuncSpec.ts";
import { FuncSpec } from "../bindings/FuncSpec.ts";
import { FuncSpecData } from "../bindings/FuncSpecData.ts";
import { FuncSpecBackendKind } from "../bindings/FuncSpecBackendKind.ts";
import { FuncSpecBackendResponseType } from "../bindings/FuncSpecBackendResponseType.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { ActionFuncSpec } from "../bindings/ActionFuncSpec.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";
import { LeafFunctionSpec } from "../bindings/LeafFunctionSpec.ts";
import { LeafKind } from "../bindings/LeafKind.ts";
import { ManagementFuncSpec } from "../bindings/ManagementFuncSpec.ts";
import { Buffer } from "node:buffer";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export interface FuncSpecInfo {
  id: string;
  backendKind: FuncSpecBackendKind;
  responseType: FuncSpecBackendResponseType;
  displayName: string;
  path: string;
}

export function createFunc(
  name: string,
  backendKind: FuncSpecBackendKind,
  responseType: FuncSpecBackendResponseType,
  codeBase64: string,
  id: string,
  args: FuncArgumentSpec[],
): FuncSpec {
  const data: FuncSpecData = {
    name,
    displayName: null,
    description: null,
    handler: "main",
    codeBase64,
    backendKind,
    responseType,
    hidden: false,
    isTransformation: false,
    link: null,
  };

  return {
    name,
    uniqueId: id,
    data,
    deleted: false,
    isFromBuiltin: null,
    arguments: args,
  };
}

export function createDefaultFuncSpec(
  name: string,
  spec: FuncSpecInfo,
  args: FuncArgumentSpec[],
): FuncSpec {
  const code = Deno.readTextFileSync(spec.path);
  const codeBase64: string = strippedBase64(code);

  return createFunc(
    name,
    spec.backendKind,
    spec.responseType,
    codeBase64,
    spec.id,
    args,
  );
}

export function createActionFuncSpec(
  kind: ActionFuncSpecKind,
  funcUniqueId: string,
): ActionFuncSpec {
  return {
    name: null,
    funcUniqueId,
    kind,
    deleted: false,
    uniqueId: null,
  };
}

export function createLeafFuncSpec(
  leafKind: LeafKind,
  funcUniqueId: string,
  inputs: ("domain" | "code")[],
): LeafFunctionSpec {
  return {
    funcUniqueId,
    deleted: false,
    inputs,
    leafKind,
    uniqueId: null,
  };
}

export function createManagementFuncSpec(
  name: string,
  funcUniqueId: string,
): ManagementFuncSpec {
  return {
    name,
    description: null,
    funcUniqueId,
  };
}

export function createAuthenticationFuncSpec(
  name: string,
  funcUniqueId: string,
): AuthenticationFuncSpec {
  return {
    name,
    deleted: false,
    funcUniqueId,
    uniqueId: null,
  };
}

export function modifyFunc(
  spec: ExpandedPkgSpec,
  targetId: string,
  newId: string,
  path: string,
) {
  const variant = spec.schemas[0].variants[0];
  const func = spec.funcs.find((f: FuncSpec) => f.uniqueId === targetId);
  const func_spec = [
    variant.actionFuncs,
    variant.leafFunctions,
    variant.managementFuncs,
  ]
    .flat()
    .find((item) => item.funcUniqueId === targetId);

  const code = Deno.readTextFileSync(path);
  const codeBase64: string = strippedBase64(code);

  if (func_spec && func) {
    func_spec.funcUniqueId = newId;
    func.uniqueId = newId;
    if (func.data) {
      func.data.codeBase64 = codeBase64;
    }
  }
}

// Si uses a version of base64 that removes the padding at the end for some reason
export function strippedBase64(code: string) {
  return Buffer.from(code).toString("base64").replace(/=*$/, "");
}
