import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

/// TODO: generate real shas for these before importing
export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "e1a2b3c4d5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a Microsoft Entra Asset",
    path: "./src/pipelines/msgraph/funcs/actions/refresh.ts",
  },
  "Create Asset": {
    id: "f2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a Microsoft Entra Asset",
    path: "./src/pipelines/msgraph/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a Microsoft Entra Asset",
    path: "./src/pipelines/msgraph/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a Microsoft Entra Asset",
    path: "./src/pipelines/msgraph/funcs/actions/delete.ts",
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const CODE_GENERATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const MANAGEMENT_FUNCS = {
  "Discover on Microsoft Entra": {
    id: "c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Microsoft Entra asset",
    path: "./src/pipelines/msgraph/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import from Microsoft Entra": {
    id: "d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a certain Microsoft Entra asset",
    path: "./src/pipelines/msgraph/funcs/management/import.ts",
    handlers: ["read"],
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { handlers: CfHandlerKind[] }
>;

export const QUALIFICATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;
