import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { createDefaultFuncSpec, FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

/**
 * Generic implementation for creating action funcs from provider spec data.
 * All providers use this same logic, just with different spec definitions.
 */
export function createActionFuncs(
  actionSpecs: Record<
    string,
    FuncSpecInfo & { actionKind: ActionFuncSpecKind }
  >,
): Array<{ spec: FuncSpec; kind: ActionFuncSpecKind }> {
  return Object.entries(actionSpecs).map(([func, spec]) => ({
    spec: createDefaultFuncSpec(func, spec, []),
    kind: spec.actionKind,
  }));
}

/**
 * Generic implementation for creating code generation funcs from provider spec data.
 * All providers use this same logic, just with different spec definitions.
 */
export function createCodeGenFuncs(
  codeGenSpecs: Record<string, FuncSpecInfo>,
  domain_id: string,
): FuncSpec[] {
  if (!domain_id) {
    throw new Error("no domain id provided for codegen func!");
  }

  return Object.entries(codeGenSpecs).map(([func, spec]) =>
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

/**
 * Generic implementation for creating management funcs from provider spec data.
 * All providers use this same logic, just with different spec definitions.
 */
export function createManagementFuncs(
  managementSpecs: Record<
    string,
    FuncSpecInfo & { handlers: CfHandlerKind[] }
  >,
): Array<{ func: FuncSpec; handlers: CfHandlerKind[] }> {
  return Object.entries(managementSpecs).map(([func, spec]) => ({
    func: createDefaultFuncSpec(func, spec, []),
    handlers: spec.handlers,
  }));
}

/**
 * Generic implementation for creating qualification funcs from provider spec data.
 * All providers use this same logic, just with different spec definitions.
 */
export function createQualificationFuncs(
  qualificationSpecs: Record<string, FuncSpecInfo>,
  domain_id: string,
): FuncSpec[] {
  if (!domain_id) {
    throw new Error("no domain id provided for qualification func!");
  }

  return Object.entries(qualificationSpecs).map(([func, spec]) =>
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
