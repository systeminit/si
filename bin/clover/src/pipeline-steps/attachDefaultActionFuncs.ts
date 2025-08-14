import _ from "lodash";
import {
  createActionFuncSpec,
  createDefaultActionFuncs,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { CfHandlerKind } from "../cfDb.ts";

export function attachDefaultActionFuncs(
  specs: readonly ExpandedPkgSpec[],
) {
  const defaultActionFuncs = createDefaultActionFuncs();

  for (const spec of specs) {
    const {
      funcs,
      schemas: [{ variants: [variant] }],
    } = spec;
    const { actionFuncs, cfSchema } = variant;

    for (const { spec: actionFunc, kind } of defaultActionFuncs) {
      // Make sure the Cloud Formation can handle the action too!
      const handlerKind: CfHandlerKind = kind === "refresh" ? "read" : kind;
      if (!cfSchema.handlers?.[handlerKind]) continue;

      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(actionFunc));
      actionFuncs.push(createActionFuncSpec(kind, actionFunc.uniqueId));
    }
  }
}
