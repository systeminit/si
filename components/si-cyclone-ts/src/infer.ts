import { InferContext, EvaluateFromResult } from "./index";
import { RegistryLookupError } from "./errors";
import { registry } from "si-registry";
import { SiEntity } from "si-entity";
import _ from "lodash";

export interface InferArgs {
  targetEntity: SiEntity;
  context: InferContext;
  // Probably something here about enabling/disabling/overriding
}

// Path to a function is entityType.PATH.function()
export function infer({
  targetEntity,
  context,
}: InferArgs): EvaluateFromResult[] {
  const schema = registry[targetEntity.entityType];
  if (!schema) {
    throw new RegistryLookupError({ entityType: targetEntity.entityType });
  }

  // Assemble array of functions to execute
  // Populate the userProvides data from Inferargs
  // Execute each function in order
  //
  for (const prop of schema.properties) {
    if (prop.inference) {
      if (
        prop.inference.select &&
        prop.inference.select.single &&
        prop.inference.select.default
      ) {
        const defaultFunction = _.find(
          prop.inference.functions,
          (f) => f.name == prop.inference.select.default,
        );
        if (defaultFunction.userProvides) {
          for (const userProvided of defaultFunction.userProvides) {
            if (userProvided.from) {
              for (const userProvidedFrom of userProvided.from) {
                //if (userProvidedFrom.kind == "entityId") {
                //}
              }
            }
          }
        }
      }
    }
  }
}
