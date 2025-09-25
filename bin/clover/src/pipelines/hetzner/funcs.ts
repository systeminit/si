import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { createDefaultFuncSpec, FuncSpecInfo } from "../../spec/funcs.ts";

export const ACTION_FUNC_SPECS = {
  // Actions
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

export const AUTHENTICATION_FUNC_SPECS = {
  "Hetzner Authentication": {
    id: "d63c1360e3b82a50d2c391b613c930fd1323dd064f0340142d962c4712e930af",
    displayName: "Authentication with Hetzner Cloud",
    path: "./src/pipelines/hetzner/funcs/authentication/authenticateHetzner.ts",
    backendKind: "jsAuthentication",
    responseType: "action",
  },
} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const CODE_GENERATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const MANAGEMENT_FUNCS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const QUALIFICATION_FUNC_SPECS = {
  "Hetzner Authentication Qualification": {
    id: "f594dc6ebe7597027203a39f2bef0307f2c09d97067c1a4e1a4fb9f7f3b9d379",
    displayName: "Qualify Credentials with Hetzner Cloud",
    path:
      "./src/pipelines/hetzner/funcs/qualifications/credentialQualification.ts",
    backendKind: "jsAttribute",
    responseType: "qualification",
  },
} as const as Record<
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
    func: createDefaultFuncSpec(func, spec as FuncSpecInfo, []),
    handlers: spec.handlers,
  }));
}
