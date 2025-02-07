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
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const funcs = spec.funcs;
    const mgmtFuncs = schemaVariant.managementFuncs;

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
