import _ from "lodash";
import { createLeafFuncSpec } from "../../spec/funcs.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";

export function generateDefaultLeafFuncs(
  specs: ExpandedPkgSpec[],
  fn: LeafFn,
): ExpandedPkgSpec[] {
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

    const defaultCodeGenFuncs = fn(domain_id);

    // Get available handlers from the schema
    const superSchema = schemaVariant.superSchema;
    const availableHandlers = superSchema.handlers
      ? Object.keys(superSchema.handlers)
      : [];

    for (const codeGenFunc of defaultCodeGenFuncs) {
      // Check if this func has required handlers
      if (codeGenFunc.requiredHandlers && codeGenFunc.requiredHandlers.length > 0) {
        // Check if all required handlers exist on this schema
        const hasAllHandlers = codeGenFunc.requiredHandlers.every(
          (required) => availableHandlers.includes(required)
        );

        if (!hasAllHandlers) {
          // Skip this codegen func for this schema
          continue;
        }
      }

      // clone otherwise modifications to these cause changes on all
      // specs
      funcs.push(_.cloneDeep(codeGenFunc.spec));
      leafFuncs.push(
        createLeafFuncSpec("codeGeneration", codeGenFunc.spec.uniqueId, ["domain"]),
      );
    }
  }

  return specs;
}

export type LeafFn = (domain_id: string) => Array<{
  spec: FuncSpec;
  requiredHandlers?: string[]
}>;
