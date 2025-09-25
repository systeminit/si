import _ from "lodash";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { CfHandlerKind } from "../types.ts";
import { createActionFuncSpec } from "../../spec/funcs.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";

export function generateDefaultActionFuncs(
  specs: ExpandedPkgSpec[],
  fn: Fn,
): ExpandedPkgSpec[] {
  const defaultActionFuncs = fn();

  for (const spec of specs) {
    const {
      funcs,
      schemas: [{ variants: [variant] }],
    } = spec;
    const { actionFuncs, superSchema } = variant;

    for (const { spec: actionFunc, kind } of defaultActionFuncs) {
      // Make sure the Cloud Formation can handle the action too!
      let handlerKind: CfHandlerKind;
      switch (kind) {
        case "refresh":
        case "other":
          handlerKind = "read";
          break;
        default:
          handlerKind = kind;
      }
      if (!superSchema.handlers?.[handlerKind]) continue;

      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(actionFunc));
      // Generic
      actionFuncs.push(createActionFuncSpec(kind, actionFunc.uniqueId));
    }
  }

  return specs;
}

export type Fn = () => { spec: FuncSpec; kind: ActionFuncSpecKind }[];

