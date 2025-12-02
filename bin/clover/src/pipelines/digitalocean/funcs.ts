import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

/// TODO: generate real shas for these before importing
export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "d1g2i3t4a5l6o7c8e9a0n1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh a DigitalOcean Asset",
    path: "./src/pipelines/digitalocean/funcs/actions/refresh.ts",
  },
  "Create Asset": {
    id: "d2g3i4t5a6l7o8c9e0a1n2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create a DigitalOcean Asset",
    path: "./src/pipelines/digitalocean/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "d3g4i5t6a7l8o9c0e1a2n3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update a DigitalOcean Asset",
    path: "./src/pipelines/digitalocean/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "d4g5i6t7a8l9o0c1e2a3n4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete a DigitalOcean Asset",
    path: "./src/pipelines/digitalocean/funcs/actions/delete.ts",
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const CODE_GENERATION_FUNC_SPECS = {
  "DigitalOcean Create Code Gen": {
    id: "d7g8i9t0a1l2o3c4e5a6n7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for creating a DigitalOcean Asset",
    path: "./src/pipelines/digitalocean/funcs/code-gen/digitalOceanCodeGenCreate.ts",
    requiredHandlers: ["create"],
  },
  "DigitalOcean Update Code Gen": {
    id: "d8g9i0t1a2l3o4c5e6a7n8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
    displayName: "Code Gen for updating a DigitalOcean Asset",
    path: "./src/pipelines/digitalocean/funcs/code-gen/digitalOceanCodeGenUpdate.ts",
    requiredHandlers: ["update"],
  },
} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const MANAGEMENT_FUNCS = {
  "Discover on DigitalOcean": {
    id: "d5g6i7t8a9l0o1c2e3a4n5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover all of a certain DigitalOcean asset",
    path: "./src/pipelines/digitalocean/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import from DigitalOcean": {
    id: "d6g7i8t9a0l1o2c3e4a5n6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7",
    backendKind: "management",
    responseType: "management",
    displayName: "Import a certain DigitalOcean asset",
    path: "./src/pipelines/digitalocean/funcs/management/import.ts",
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
