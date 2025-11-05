import _ from "lodash";
import { createLeafFuncSpec } from "../../spec/funcs.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";

export function generateDefaultQualificationFuncs(
  specs: ExpandedPkgSpec[],
  fn: QualificationFn,
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

    const defaultQualificationFuncs = fn(domain_id);

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

    newSpecs.push(spec);
  }

  return newSpecs;
}

export type QualificationFn = (domain_id: string) => FuncSpec[];
