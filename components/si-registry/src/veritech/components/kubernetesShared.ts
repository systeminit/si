import { Entity, CalculatePropertiesResult } from "../intelligence";
import _ from "lodash";

export function kubernetesNamespaceProperties(
  result: CalculatePropertiesResult,
  namespace: Entity,
): void {
  if (namespace.properties.__baseline.kubernetesObject?.metadata?.name) {
    _.set(
      result.inferredProperties,
      ["__baseline", "kubernetesObject", "metadata", "namespace"],
      namespace.properties.__baseline.kubernetesObject.metadata.name,
    );
  }
}
