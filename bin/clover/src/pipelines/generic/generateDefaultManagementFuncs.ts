import _ from "lodash";
import { createManagementFuncSpec } from "../../spec/funcs.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { CfHandlerKind } from "../types.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";

export function generateDefaultManagementFuncs(
  specs: ExpandedPkgSpec[],
  fn: ManagementFn,
): ExpandedPkgSpec[] {
  const defaultMgmtFuncs = fn();

  for (const spec of specs) {
    const { funcs, schemas: [{ variants: [variant] }] } = spec;
    const { superSchema } = variant;

    for (const { func, handlers } of defaultMgmtFuncs) {
      // Skip management funcs that require handlers we don't have
      if (!handlers.every((handler) => superSchema.handlers?.[handler])) {
        continue;
      }

      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(func));
      variant.managementFuncs.push(
        createManagementFuncSpec(func.name, func.uniqueId),
      );
    }
  }

  return specs;
}

export type ManagementFn = () => {
  func: FuncSpec;
  handlers: CfHandlerKind[];
}[];
