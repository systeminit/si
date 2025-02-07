import _ from "lodash";
import {
  createDefaultCodeGenFuncs,
  createLeafFuncSpec,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateDefaultLeafFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];
    const funcs = spec.funcs;
    const leafFuncs = schemaVariant.leafFunctions;
    const domain_id = schemaVariant.domain.uniqueId;

    if (!schemaVariant || !domain_id) {
      console.log(
        `Could not generate action funcs for ${spec.name}: missing schema, variant, or domain id!`,
      );
      continue;
    }

    const defaultCodeGenFuncs = createDefaultCodeGenFuncs(domain_id);

    for (const codeGenFunc of defaultCodeGenFuncs) {
      funcs.push(codeGenFunc);
      leafFuncs.push(
        createLeafFuncSpec(
          "codeGeneration",
          codeGenFunc.uniqueId,
        ),
      );
    }

    newSpecs.push(spec);
  }

  return newSpecs;
}
