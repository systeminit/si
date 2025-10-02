import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

export const ACTION_FUNC_SPECS = {
  // Actions
  "Create Asset": {
    id: "bc58dae4f4e1361840ec8f081350d7ec6b177ee8dc5a6a55155767c92efe1850",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/actions/awsCloudControlCreate.ts",
  },
  "Delete Asset": {
    id: "8987ae62887646ccb55303cf82d69364dadd07a817580b57cb292988a64867d1",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/actions/awsCloudControlDelete.ts",
  },
  "Refresh Asset": {
    id: "b161c855b802f5da7d96fb8cbda0195170d4769da9953cc5b5fb352fd441628c",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/actions/awsCloudControlRefresh.ts",
  },
  "Update Asset": {
    id: "125cf080759938d293470dfe97268e1600375bb3e22fc162d1a3b5bb0f18a67b",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/actions/awsCloudControlUpdate.ts",
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
    path: "./src/pipelines/aws/funcs/code-gen/awsCloudControlCodeGenCreate.ts",
  },
  awsCloudControlUpdate: {
    id: "f170263ef3fbb0e8017b47221c5d70ae412b2eaa33e75e1a98525c9a070d60f6",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for updating a Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/code-gen/awsCloudControlCodeGenUpdate.ts",
  },
  awsCloudFormationLint: {
    id: "fdef639540613ce1639df4153f9bb5a8929e9815477eb57beb3616af48a74335",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for use in validating a Cloudformation document",
    path: "./src/pipelines/aws/funcs/code-gen/awsCloudFormationLint.ts",
  },
} as const satisfies Record<string, FuncSpecInfo>;

export const MANAGEMENT_FUNCS = {
  // Management
  "Discover on AWS": {
    id: "dba1f6e327c1e82363fa3ceaf0d3e908d367ed7c6bfa25da0a06127fb81ff1b6",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/management/awsCloudControlDiscover.ts",
    handlers: ["list", "read"],
  },
  "Import from AWS": {
    id: "7a8dfabe771e66d13ccd02376eee84979fbc2f2974f86b60f8710c6db24122c6",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a Cloud Control Asset",
    path: "./src/pipelines/aws/funcs/management/awsCloudControlImport.ts",
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

