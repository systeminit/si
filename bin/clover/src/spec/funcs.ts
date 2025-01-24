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

interface FuncSpecInfo {
  id: string;
  backendKind: FuncSpecBackendKind;
  responseType: FuncSpecBackendResponseType;
  displayName: string;
  actionKind?: "create" | "update" | "refresh" | "other" | "delete";
  path: string;
}

const funcSpecs: Record<string, FuncSpecInfo> = {
  // Actions
  "Create Asset": {
    id: "bc58dae4f4e1361840ec8f081350d7ec6b177ee8dc5a6a55155767c92efe1850",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlCreate.ts",
  },
  "Delete Asset": {
    id: "8987ae62887646ccb55303cf82d69364dadd07a817580b57cb292988a64867d1",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlDelete.ts",
  },
  "Refresh Asset": {
    id: "b161c855b802f5da7d96fb8cbda0195170d4769da9953cc5b5fb352fd441628c",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlRefresh.ts",
  },
  "Update Asset": {
    id: "125cf080759938d293470dfe97268e1600375bb3e22fc162d1a3b5bb0f18a67b",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a Cloud Control Asset",
    path: "./src/cloud-control-funcs/actions/awsCloudControlUpdate.ts",
  },
  // Code Generation
  "awsCloudControlCreate": {
    id: "c48518d82a2db7064e7851c36636c665dce775610d08958a8a4f0c5c85cd808e",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for creating a Cloud Control Asset",
    path: "./src/cloud-control-funcs/code-gen/awsCloudControlCodeGenCreate.ts",
  },
  "awsCloudControlUpdate": {
    id: "f170263ef3fbb0e8017b47221c5d70ae412b2eaa33e75e1a98525c9a070d60f6",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for updating a Cloud Control Asset",
    path: "./src/cloud-control-funcs/code-gen/awsCloudControlCodeGenUpdate.ts",
  },
  // Management
  "Discover on AWS": {
    id: "dba1f6e327c1e82363fa3ceaf0d3e908d367ed7c6bfa25da0a06127fb81ff1b6",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Cloud Control Asset",
    path: "./src/cloud-control-funcs/management/awsCloudControlDiscover.ts",
  },
  "Import from AWS": {
    id: "7a8dfabe771e66d13ccd02376eee84979fbc2f2974f86b60f8710c6db24122c6",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a Cloud Control Asset",
    path: "./src/cloud-control-funcs/management/awsCloudControlImport.ts",
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

function createDefaultFuncSpec(
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

export function createDefaultActionFuncs(): {
  spec: FuncSpec;
  kind: string;
}[] {
  const ret = [];
  const actionFuncs = [
    "Create Asset",
    "Delete Asset",
    "Refresh Asset",
    "Update Asset",
  ];

  for (const func of actionFuncs) {
    const spec = funcSpecs[func];
    if (!spec.actionKind) {
      throw new Error(`${func} spec does not have an action type`);
    }

    ret.push({
      spec: createDefaultFuncSpec(func, spec, []),
      kind: spec.actionKind,
    });
  }

  return ret;
}

export function createDefaultCodeGenFuncs(domain_id: string): FuncSpec[] {
  if (!domain_id) {
    throw new Error("no domain id provided for codegen func!");
  }

  const ret: FuncSpec[] = [];
  const codeGenFuncs = [
    "awsCloudControlCreate",
    "awsCloudControlUpdate",
  ];

  const args: FuncArgumentSpec[] = [{
    name: "domain",
    kind: "object",
    elementKind: null,
    uniqueId: domain_id,
    deleted: false,
  }];

  for (const func of codeGenFuncs) {
    const spec = funcSpecs[func];
    ret.push(createDefaultFuncSpec(func, spec, args));
  }

  return ret;
}

export function createDefaultManagementFuncs(): FuncSpec[] {
  const ret: FuncSpec[] = [];
  const actionFuncs = [
    "Discover on AWS",
    "Import from AWS",
  ];

  for (const func of actionFuncs) {
    const spec = funcSpecs[func];
    ret.push(createDefaultFuncSpec(func, spec, []));
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
    inputs: ["domain"],
    leafKind,
    uniqueId: null,
  };
}

export function createManagementFuncSpec(
  name: string,
  id: string,
): ManagementFuncSpec {
  return {
    name,
    description: null,
    funcUniqueId: id,
    managedSchemas: null,
  };
}

// Si uses a version of base64 that removes the padding at the end for some reason
export function strippedBase64(code: string) {
  return btoa(
    code,
  ).replace(/=/g, "");
}
