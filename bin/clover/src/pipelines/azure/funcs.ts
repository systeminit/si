import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/refresh.ts",
  },
  "Create Asset": {
    id: "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/delete.ts",
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const AUTHENTICATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const CODE_GENERATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const MANAGEMENT_FUNCS = {
  "Discover on Azure": {
    id: "e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Azure asset",
    path: "./src/pipelines/azure/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import from Azure": {
    id: "f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a certain Azure asset",
    path: "./src/pipelines/azure/funcs/management/import.ts",
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
