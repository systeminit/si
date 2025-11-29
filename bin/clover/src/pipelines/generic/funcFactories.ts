import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { FuncArgumentSpec } from "../../bindings/FuncArgumentSpec.ts";
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

/**
 * Generic implementation for creating attribute funcs from provider spec data.
 * Attribute functions compute or query values based on input bindings.
 * All providers use this same logic, just with different spec definitions.
 *
 * @param attributeSpecs - Map of function names to their specs
 * @param bindings - Function argument bindings. Can be:
 *   - FuncArgumentSpec[] - Same bindings for all functions
 *   - Record<string, FuncArgumentSpec[]> - Custom bindings per function name
 *   - (funcName: string) => FuncArgumentSpec[] - Generator function
 */
export function createAttributeFuncs(
  attributeSpecs: Record<string, FuncSpecInfo>,
  bindings:
    | FuncArgumentSpec[]
    | Record<string, FuncArgumentSpec[]>
    | ((funcName: string) => FuncArgumentSpec[]),
): FuncSpec[] {
  return Object.entries(attributeSpecs).map(([funcName, spec]) => {
    let funcBindings: FuncArgumentSpec[];

    if (typeof bindings === "function") {
      funcBindings = bindings(funcName);
    } else if (Array.isArray(bindings)) {
      funcBindings = bindings;
    } else {
      funcBindings = bindings[funcName] || [];
    }

    return createDefaultFuncSpec(funcName, spec, funcBindings);
  });
}
