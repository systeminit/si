import { FuncSpec } from "../bindings/FuncSpec.ts";
import { FuncSpecData } from "../bindings/FuncSpecData.ts";
import { FuncSpecBackendKind } from "../bindings/FuncSpecBackendKind.ts";
import { FuncSpecBackendResponseType } from "../bindings/FuncSpecBackendResponseType.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { ActionFuncSpec } from "../bindings/ActionFuncSpec.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";
import { LeafFunctionSpec } from "../bindings/LeafFunctionSpec.ts";
import { LeafKind } from "../bindings/LeafKind.ts";

interface FuncSpecInfo {
  id: string;
  backendKind: FuncSpecBackendKind;
  responseType: FuncSpecBackendResponseType;
  displayName: string;
  path: string;
}

const funcSpecs: Record<string, FuncSpecInfo> = {
  // Actions
  "create": {
    id: "bc58dae4f4e1361840ec8f081350d7ec6b177ee8dc5a6a55155767c92efe1850",
    backendKind: "jsAction",
    responseType: "action",
    displayName: "Create a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlCreate.ts",
  },
  "delete": {
    id: "8987ae62887646ccb55303cf82d69364dadd07a817580b57cb292988a64867d1",
    backendKind: "jsAction",
    responseType: "action",
    displayName: "Delete a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlDelete.ts",
  },
  "refresh": {
    id: "b161c855b802f5da7d96fb8cbda0195170d4769da9953cc5b5fb352fd441628c",
    backendKind: "jsAction",
    responseType: "action",
    displayName: "Refresh a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlRefresh.ts",
  },
  "update": {
    id: "125cf080759938d293470dfe97268e1600375bb3e22fc162d1a3b5bb0f18a67b",
    backendKind: "jsAction",
    responseType: "action",
    displayName: "Update a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlUpdate.ts",
  },
  // Code Generation
  "codeGenCreate": {
    id: "c48518d82a2db7064e7851c36636c665dce775610d08958a8a4f0c5c85cd808e",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for creating a Cloud Control Asset",
    path: "./src/cloud-control-funcs/code-gen/awsCloudControlCodeGenCreate.ts",
  },
  "codeGenUpdate": {
    id: "f170263ef3fbb0e8017b47221c5d70ae412b2eaa33e75e1a98525c9a070d60f6",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for updating a Cloud Control Asset",
    path: "./src/cloud-control-funcs/code-gen/awsCloudControlCodeGenUpdate.ts",
  },
};

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

function createDefaultFuncSpec(name: string, args: FuncArgumentSpec[]): FuncSpec {
  const spec = funcSpecs[name];
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

export function createDefaultActionFuncs(): FuncSpec[] {
  const ret: FuncSpec[] = [];
  const actionFuncs = [
    "create",
    "delete",
    "refresh",
    "update",
  ];

  for (const func of actionFuncs) {
    ret.push(createDefaultFuncSpec(func, []));
  }

  return ret;
}

export function createDefaultCodeGenFuncs(domain_id: string): FuncSpec[] {
  if (!domain_id) {
    throw new Error("no domain id provided for codegen func!")
  }

  const ret: FuncSpec[] = [];
  const codeGenFuncs = [
    "codeGenCreate",
    "codeGenUpdate",
  ];

  const args: FuncArgumentSpec[] = [{
    name: "domain",
    kind: "object",
    elementKind: null,
    uniqueId: domain_id,
    deleted: false
  }]

  for (const func of codeGenFuncs) {
    ret.push(createDefaultFuncSpec(func, args));
  }

  return ret;
}

export function createActionFuncSpec(
  kind: ActionFuncSpecKind,
  id: string,
): ActionFuncSpec {
  return {
    name: null,
    funcUniqueId: id,
    kind,
    deleted: false,
    uniqueId: null,
  };
}

export function createLeafFuncSpec(
  leafKind: LeafKind,
  id: string,
): LeafFunctionSpec {
  return {
    funcUniqueId: id,
    deleted: false,
    inputs: [ "domain" ],
    leafKind,
    uniqueId: null,
  };
}

// Si uses a version of base64 that removes the padding at the end for some reason
export function strippedBase64(code: string) {
  return btoa(
    code,
  ).replace(/=/g, "");
}
