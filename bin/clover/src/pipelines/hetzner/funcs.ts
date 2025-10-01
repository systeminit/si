import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "dacba8b3e1138c7d6beb2d5236db376cfdf318a51f31d207c159859aa6984e22",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a Hetzner Asset",
    path: "./src/pipelines/hetzner/funcs/actions/refresh.ts",
  },
  "Create Asset": {
    id: "f5c7e2a8b9d3f4e1a6c2b8d9e3f7a4b1c6d2e8f9a3b7c1d5e9f2a6b8c4d7e1f3",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a Hetzner Asset",
    path: "./src/pipelines/hetzner/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "a8b4c7d1e9f3a6b2c5d8e4f7a1b9c3d6e2f8a5b7c9d4e1f6a3b8c2d7e9f4a6b1",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a Hetzner Asset",
    path: "./src/pipelines/hetzner/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "b9c5d8e1f4a7b3c6d9e2f5a8b1c4d7e0f3a6b9c2d5e8f1a4b7c0d3e6f9a2b5c8",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a Hetzner Asset",
    path: "./src/pipelines/hetzner/funcs/actions/delete.ts",
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
  "Discover on Hetzner": {
    id: "f92d6e959c63b34406ad8c23444edf15a2ec4be3380afcd6a12136f33ce65e9b",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain Hetzner asset",
    path: "./src/pipelines/hetzner/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import from Hetzner": {
    id: "3f99f0c682e7bd0b82d0533d170e9475a38a64f86bfe73bbe17b15abf8773d58",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a certain Hetzner asset",
    path: "./src/pipelines/hetzner/funcs/management/import.ts",
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

