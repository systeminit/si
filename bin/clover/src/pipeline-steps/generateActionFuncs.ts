import _ from "lodash";
import {
  createActionFuncSpec,
  createDefaultActionFuncs,
} from "../spec/funcs.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateDefaultActionFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];
  const defaultActionFuncs = createDefaultActionFuncs();

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];
    const funcs = spec.funcs;
    const actionFuncs = schemaVariant.actionFuncs;

    if (!schemaVariant) {
      console.log(
        `Could not generate action funcs for ${spec.name}: missing schema or variant!`,
      );
      continue;
    }

    for (const { spec: actionFunc, kind } of defaultActionFuncs) {
      funcs.push(actionFunc);
      actionFuncs.push(
        createActionFuncSpec(
          kind as ActionFuncSpecKind,
          actionFunc.uniqueId,
        ),
      );
    }

    newSpecs.push(spec);
  }

  return newSpecs;
}
