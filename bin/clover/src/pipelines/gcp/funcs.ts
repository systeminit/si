// NOTE: Function IDs are fake SHAs - update with real SHAs when functions are implemented

import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "0000000000000000000000000000000000000001",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a GCP Asset",
    path: "./src/pipelines/gcp/funcs/actions/refresh.ts",
  },
  "Create Asset": {
    id: "0000000000000000000000000000000000000002",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a GCP Asset",
    path: "./src/pipelines/gcp/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "0000000000000000000000000000000000000003",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a GCP Asset",
    path: "./src/pipelines/gcp/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "0000000000000000000000000000000000000004",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a GCP Asset",
    path: "./src/pipelines/gcp/funcs/actions/delete.ts",
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
  "Discover on GCP": {
    id: "0000000000000000000000000000000000000006",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain GCP asset",
    path: "./src/pipelines/gcp/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import from GCP": {
    id: "0000000000000000000000000000000000000007",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a certain GCP asset",
    path: "./src/pipelines/gcp/funcs/management/import.ts",
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
