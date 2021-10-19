import Debug from "debug";
const debug = Debug("veritech:controllers:intel:k8sSecret");
import {
  baseCheckQualifications,
  baseRunCommands,
  baseSyncResource,
} from "./k8sShared";

import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";

import {
  setProperty,
  setPropertyFromEntity,
  setPropertyFromEntitySecret,
  setPropertyFromProperty,
} from "./inferShared";

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

  setPropertyFromProperty({
    entity,
    fromPath: ["metadata", "name"],
    toPath: ["metadata", "labels", "app"],
  });

  // Do you have a k8s namespace? If so, set the namespace.
  setPropertyFromEntity({
    context,
    entityType: "k8sNamespace",
    fromPath: ["metadata", "name"],
    toEntity: entity,
    toPath: ["metadata", "namespace"],
  });

  // The template should have a namespace that matches the namespace of the
  // object we are deploying.
  setPropertyFromProperty({
    entity,
    fromPath: ["metadata", "namespace"],
    toPath: ["metadata", "namespace"],
  });

  setPropertyFromEntity({
    context,
    entityType: "dockerHubCredential",
    fromPath: ["secret"],
    toEntity: entity,
    toPath: ["type"],
    transform() {
      return "kubernetes.io/dockerconfigjson";
    },
  });

  setPropertyFromEntitySecret({
    context,
    entityType: "dockerHubCredential",
    toEntity: entity,
    toPath: ["data", ".dockerconfigjson"],
    transform(decrypted) {
      const auth = Buffer.from(
        `${decrypted["username"]}:${decrypted["password"]}`,
      ).toString("base64");
      const config = {
        auths: {
          "https://index.docker.io/v1/": {
            auth,
          },
        },
      };
      const base64d = Buffer.from(JSON.stringify(config, null, 0)).toString(
        "base64",
      );

      return base64d;
    },
  });

  return { entity };
}

export default {
  inferProperties,
  checkQualifications: baseCheckQualifications,
  runCommands: baseRunCommands,
  syncResource: baseSyncResource,
};
