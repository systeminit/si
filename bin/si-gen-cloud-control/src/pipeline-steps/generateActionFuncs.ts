import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "lodash";
import {
  createActionFuncSpec,
  createDefaultActionFuncs,
} from "../spec/funcs.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";

export function generateDefaultActionFuncs(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];
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

    for (const actionFunc of defaultActionFuncs) {
      funcs.push(actionFunc);
      actionFuncs.push(
        createActionFuncSpec(
          actionFunc.name as ActionFuncSpecKind,
          actionFunc.uniqueId,
        ),
      );
    }

    newSpecs.push(spec);
  }

  return newSpecs;
}
