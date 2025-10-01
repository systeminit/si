import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

export const ACTION_FUNC_SPECS = {
  "Create Asset": {
    id: "dummy0001000000000000000000000000000000000000000000000000000000000001",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a Dummy Asset",
    path: "./src/pipelines/dummy/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "dummy0002000000000000000000000000000000000000000000000000000000000002",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a Dummy Asset",
    path: "./src/pipelines/dummy/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "dummy0003000000000000000000000000000000000000000000000000000000000003",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a Dummy Asset",
    path: "./src/pipelines/dummy/funcs/actions/delete.ts",
  },
  "Refresh Asset": {
    id: "dummy0004000000000000000000000000000000000000000000000000000000000004",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a Dummy Asset",
    path: "./src/pipelines/dummy/funcs/actions/refresh.ts",
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
  "Discover Dummy Assets": {
    id: "dummy0005000000000000000000000000000000000000000000000000000000000005",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all Dummy assets",
    path: "./src/pipelines/dummy/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import Dummy Asset": {
    id: "dummy0006000000000000000000000000000000000000000000000000000000000006",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a Dummy asset",
    path: "./src/pipelines/dummy/funcs/management/import.ts",
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

