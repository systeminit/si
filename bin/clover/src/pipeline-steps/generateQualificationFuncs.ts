import _ from "lodash";
import {
  createDefaultQualificationFuncs,
  createLeafFuncSpec,
} from "../spec/funcs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateDefaultQualificationFuncs(
  specs: readonly ExpandedPkgSpec[],
) {
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const funcs = spec.funcs;
    const leafFuncs = schemaVariant.leafFunctions;
    const domain_id = schemaVariant.domain.uniqueId;

    if (!domain_id) {
      throw new Error(
        `Could not generate qualification funcs for ${spec.name}: missing domain id!`,
      );
    }

    const defaultQualificationFuncs = createDefaultQualificationFuncs(
      domain_id,
    );

    for (const func of defaultQualificationFuncs) {
      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(func));
      leafFuncs.push(
        createLeafFuncSpec("qualification", func.uniqueId, [
          "domain",
          "code",
        ]),
      );
    }
  }
}
