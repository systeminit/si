import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import { setProperty, setPropertyFromEntity } from "./inferShared";
import {
  baseCheckQualifications,
  baseRunCommands,
  baseSyncResource,
} from "./k8sShared";

export function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const context = request.context;
  const entity = request.entity;

  setProperty({
    entity,
    toPath: ["metadata", "name"],
    value: entity.name,
  });

  // Do you have a k8s namespace? If so, set the namespace.
  setPropertyFromEntity({
    context,
    entityType: "k8sNamespace",
    fromPath: ["metadata", "name"],
    toEntity: entity,
    toPath: ["metadata", "namespace"],
  });

  return { entity };
}

export default {
  inferProperties,
  checkQualifications: baseCheckQualifications,
  runCommands: baseRunCommands,
  syncResource: baseSyncResource,
};
