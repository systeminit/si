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
import { CfHandlerKind } from "../cfDb.ts";
import { ExpandedPkgSpec } from "./pkgs.ts";

interface FuncSpecInfo {
  id: string;
  backendKind: FuncSpecBackendKind;
  responseType: FuncSpecBackendResponseType;
  displayName: string;
  path: string;
}

export const ACTION_FUNC_SPECS = {
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
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const CODE_GENERATION_FUNC_SPECS = {
  // Code Generation
  awsCloudControlCreate: {
    id: "c48518d82a2db7064e7851c36636c665dce775610d08958a8a4f0c5c85cd808e",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for creating a Cloud Control Asset",
    path: "./src/cloud-control-funcs/code-gen/awsCloudControlCodeGenCreate.ts",
  },
  awsCloudControlUpdate: {
    id: "f170263ef3fbb0e8017b47221c5d70ae412b2eaa33e75e1a98525c9a070d60f6",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for updating a Cloud Control Asset",
    path: "./src/cloud-control-funcs/code-gen/awsCloudControlCodeGenUpdate.ts",
  },
  awsCloudFormationLint: {
    id: "fdef639540613ce1639df4153f9bb5a8929e9815477eb57beb3616af48a74335",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for use in validating a Cloudformation document",
    path: "./src/cloud-control-funcs/code-gen/awsCloudFormationLint.ts",
  },
} as const satisfies Record<string, FuncSpecInfo>;

export const MANAGEMENT_FUNCS = {
  // Management
  "Discover on AWS": {
    id: "dba1f6e327c1e82363fa3ceaf0d3e908d367ed7c6bfa25da0a06127fb81ff1b6",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Cloud Control Asset",
    path: "./src/cloud-control-funcs/management/awsCloudControlDiscover.ts",
    handlers: ["list", "read"],
  },
  "Import from AWS": {
    id: "7a8dfabe771e66d13ccd02376eee84979fbc2f2974f86b60f8710c6db24122c6",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a Cloud Control Asset",
    path: "./src/cloud-control-funcs/management/awsCloudControlImport.ts",
    handlers: ["read"],
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { handlers: CfHandlerKind[] }
>;

export const QUALIFICATION_FUNC_SPECS = {} as const as Record<
  string,
  FuncSpecInfo
>;

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

export function createDefaultActionFuncs() {
  return Object.entries(ACTION_FUNC_SPECS).map(([func, spec]) => ({
    spec: createDefaultFuncSpec(func, spec, []),
    kind: spec.actionKind,
  }));
}

export function createDefaultCodeGenFuncs(domain_id: string): FuncSpec[] {
  if (!domain_id) {
    throw new Error("no domain id provided for codegen func!");
  }

  return Object.entries(CODE_GENERATION_FUNC_SPECS).map(([func, spec]) =>
    createDefaultFuncSpec(func, spec, [
      {
        name: "domain",
        kind: "object",
        elementKind: null,
        uniqueId: domain_id,
        deleted: false,
      },
    ]),
  );
}

export function createDefaultQualificationFuncs(domain_id: string): FuncSpec[] {
  if (!domain_id) {
    throw new Error("no domain id provided for qualification func!");
  }

  return Object.entries(QUALIFICATION_FUNC_SPECS).map(([func, spec]) =>
    createDefaultFuncSpec(func, spec, [
      {
        name: "domain",
        kind: "object",
        elementKind: null,
        uniqueId: domain_id,
        deleted: false,
      },
    ]),
  );
}

export function createDefaultManagementFuncs() {
  return Object.entries(MANAGEMENT_FUNCS).map(([func, spec]) => ({
    func: createDefaultFuncSpec(func, spec, []),
    handlers: spec.handlers,
  }));
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

  func_spec.funcUniqueId = newId;
  func.uniqueId = newId;
  func.data.codeBase64 = codeBase64;
}

// Si uses a version of base64 that removes the padding at the end for some reason
export function strippedBase64(code: string) {
  return Buffer.from(code).toString("base64").replace(/=*$/, "");
}
