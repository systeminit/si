import _ from "lodash";
import {
  createDefaultManagementFuncs,
  createManagementFuncSpec,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function attachDefaultManagementFuncs(
  specs: readonly ExpandedPkgSpec[],
) {
  const defaultMgmtFuncs = createDefaultManagementFuncs();

  for (const spec of specs) {
    const { funcs, schemas: [{ variants: [variant] }] } = spec;
    const { cfSchema } = variant;

    for (const { func, handlers } of defaultMgmtFuncs) {
      // Skip management funcs that require handlers we don't have
      if (!handlers.every((handler) => cfSchema.handlers?.[handler])) continue;

      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(func));
      variant.managementFuncs.push(
        createManagementFuncSpec(func.name, func.uniqueId),
      );
    }
  }
}
