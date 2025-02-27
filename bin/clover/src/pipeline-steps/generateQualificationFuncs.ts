import _ from "lodash";
import {
  createDefaultQualificationFuncs,
  createLeafFuncSpec,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateDefaultQualificationFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const funcs = spec.funcs;
    const leafFuncs = schemaVariant.leafFunctions;
    const domain_id = schemaVariant.domain.uniqueId;

    if (!domain_id) {
      console.log(
        `Could not generate qualification funcs for ${spec.name}: missing domain id!`,
      );
      continue;
    }

    const defaultCodeGenFuncs = createDefaultQualificationFuncs(domain_id);

    for (const codeGenFunc of defaultCodeGenFuncs) {
      funcs.push(codeGenFunc);
      leafFuncs.push(
        createLeafFuncSpec("qualification", codeGenFunc.uniqueId, [
          "domain",
          "code",
        ]),
      );
    }

    newSpecs.push(spec);
  }

  return newSpecs;
}
