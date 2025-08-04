import _ from "lodash";
import {
  createDefaultCodeGenFuncs,
  createLeafFuncSpec,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateDefaultLeafFuncs(
  specs: readonly ExpandedPkgSpec[],
) {
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const funcs = spec.funcs;
    const leafFuncs = schemaVariant.leafFunctions;
    const domain_id = schemaVariant.domain.uniqueId;

    if (!domain_id) {
      console.log(
        `Could not generate codegen funcs for ${spec.name}: missing domain id!`,
      );
      continue;
    }

    const defaultCodeGenFuncs = createDefaultCodeGenFuncs(domain_id);

    for (const codeGenFunc of defaultCodeGenFuncs) {
      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(codeGenFunc));
      leafFuncs.push(
        createLeafFuncSpec("codeGeneration", codeGenFunc.uniqueId, ["domain"]),
      );
    }
  }
}
