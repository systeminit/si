import _ from "lodash";
import {
  createDefaultManagementFuncs,
  createManagementFuncSpec,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateDefaultManagementFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];
  const defaultMgmtFuncs = createDefaultManagementFuncs();

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];
    const funcs = spec.funcs;
    const mgmtFuncs = schemaVariant.managementFuncs;

    if (!schemaVariant) {
      console.log(
        `Could not generate action funcs for ${spec.name}: missing schema or variant!`,
      );
      continue;
    }

    for (const mgmtFunc of defaultMgmtFuncs) {
      funcs.push(mgmtFunc);
      mgmtFuncs.push(
        createManagementFuncSpec(
          mgmtFunc.name,
          mgmtFunc.uniqueId,
        ),
      );
    }

    newSpecs.push(spec);
  }

  return newSpecs;
}
