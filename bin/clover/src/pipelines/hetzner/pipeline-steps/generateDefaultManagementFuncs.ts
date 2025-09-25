import _ from "lodash";
import { createManagementFuncSpec } from "../../../spec/funcs.ts";
import { createDefaultManagementFuncs } from "../funcs.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";

export function generateDefaultManagementFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const defaultMgmtFuncs = createDefaultManagementFuncs();

  for (const spec of specs) {
    const { funcs, schemas: [{ variants: [variant] }] } = spec;

    for (const { func } of defaultMgmtFuncs) {
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
