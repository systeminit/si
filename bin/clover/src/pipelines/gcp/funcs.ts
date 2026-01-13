import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

const GCP_SHARED_UTILS_PATH = "./src/pipelines/gcp/funcs/shared/utils.ts";

export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "7fe9fdf099b1f9b3d3fe8ec5c88fb5bfc5c5847091e1df34c0422ff515adfcfb",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a Google Cloud Asset",
    path: "./src/pipelines/gcp/funcs/actions/refresh.ts",
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
  "Create Asset": {
    id: "6eb861eec5e5be2c4ec052713e025867cb087ff7aab730fa33e7075b62056671",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a Google Cloud Asset",
    path: "./src/pipelines/gcp/funcs/actions/create.ts",
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
  "Update Asset": {
    id: "609f5df4a00ad92d7bcd4c082b8c08632e523e78beafcc829d517f49c4ea1000",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a Google Cloud Asset",
    path: "./src/pipelines/gcp/funcs/actions/update.ts",
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
  "Delete Asset": {
    id: "17fcce87ad2f8dd41974c7ee484a458f7cb522d63c076efa93a83251e80ab716",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a Google Cloud Asset",
    path: "./src/pipelines/gcp/funcs/actions/delete.ts",
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const CODE_GENERATION_FUNC_SPECS = {
  "Google Cloud Create Code Gen": {
    id: "eeda010e1878a0d66e4564852ba47a9831809d806d7b154648fbe6eea34af955",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Google Cloud Create Codegen",
    path: "./src/pipelines/gcp/funcs/code-gen/gcpCodeGenCreate.ts",
    requiredHandlers: ["create"],
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
  "Google Cloud Update Code Gen": {
    id: "be0f9a25f4c489454d4f898902f52b63ddde7e0627300580d3fc436dc969a9cb",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Google Cloud Update Codegen",
    path: "./src/pipelines/gcp/funcs/code-gen/gcpCodeGenUpdate.ts",
    requiredHandlers: ["update"],
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const MANAGEMENT_FUNCS = {
  "Discover on Google Cloud": {
    id: "5db11619c4839dc5558fc1f46ce38cf1b8d757f18c8a756b2109ddd98cef0628",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Google Cloud asset",
    path: "./src/pipelines/gcp/funcs/management/discover.ts",
    handlers: ["list", "read"],
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
  "Import from Google Cloud": {
    id: "81757f0ff27d1fb6394c39339616682ea8689982529263f6194535a13d0474cc",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a certain Google Cloud asset",
    path: "./src/pipelines/gcp/funcs/management/import.ts",
    handlers: ["read"],
    sharedUtilsPath: GCP_SHARED_UTILS_PATH,
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { handlers: CfHandlerKind[] }
>;

export const QUALIFICATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;
