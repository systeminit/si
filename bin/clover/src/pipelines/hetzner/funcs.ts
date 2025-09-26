import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { createDefaultFuncSpec, FuncSpecInfo } from "../../spec/funcs.ts";
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
    createDefaultFuncSpec(func, spec as FuncSpecInfo, [
      {
        name: "domain",
        kind: "object",
        elementKind: null,
        uniqueId: domain_id,
        deleted: false,
      },
    ])
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
    ])
  );
}

export function createDefaultManagementFuncs() {
  return Object.entries(MANAGEMENT_FUNCS).map(([func, spec]) => ({
    func: createDefaultFuncSpec(func, spec, []),
    handlers: spec.handlers,
  }));
}
