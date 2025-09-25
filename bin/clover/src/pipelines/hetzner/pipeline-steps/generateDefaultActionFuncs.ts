import _ from "lodash";
import { createDefaultActionFuncs } from "../funcs.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import { createActionFuncSpec } from "../../../spec/funcs.ts";

export function generateDefaultActionFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const defaultActionFuncs = createDefaultActionFuncs();

  for (const spec of specs) {
    const {
      funcs,
      schemas: [{ variants: [variant] }],
    } = spec;
    const { actionFuncs } = variant;

    for (const { spec: actionFunc, kind } of defaultActionFuncs) {
      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(actionFunc));
      actionFuncs.push(createActionFuncSpec(kind, actionFunc.uniqueId));
    }
  }

  return specs;
}
