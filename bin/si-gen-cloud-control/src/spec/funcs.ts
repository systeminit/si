import { FuncSpec } from "../bindings/FuncSpec.ts";
import { FuncSpecData } from "../bindings/FuncSpecData.ts";
import { FuncSpecBackendKind } from "../bindings/FuncSpecBackendKind.ts";
import { FuncSpecBackendResponseType } from "../bindings/FuncSpecBackendResponseType.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { ActionFuncSpec } from "../bindings/ActionFuncSpec.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";

interface FuncSpecInfo {
  id: string;
  backendKind: FuncSpecBackendKind;
  responseType: FuncSpecBackendResponseType;
  displayName: string;
  path: string;
}

const funcSpecs: Record<string, FuncSpecInfo> = {
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

function createDefaultFuncSpecs(name: string): FuncSpec {
  const spec = funcSpecs[name];
  const code = Deno.readTextFileSync(spec.path);
  const codeBase64: string = strippedBase64(code);

  const args: FuncArgumentSpec[] = [];
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
    ret.push(createDefaultFuncSpecs(func));
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

// Si uses a version of base64 that removes the padding at the end for some reason
export function strippedBase64(code: string) {
  return btoa(
    code,
  ).replace(/=/g, "");
}
