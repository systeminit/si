import { OpSource, SiEntity } from "si-entity/dist/siEntity";
import Debug from "debug";
const debug = Debug("veritech:controllers:intel:k8sService");
import {
  baseInferProperties,
  baseCheckQualifications,
  baseRunCommands,
} from "./k8sShared";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import {
  allEntitiesByType,
  findProperty,
  SetArrayEntryFromAllEntities,
  setArrayEntryFromAllEntities,
  setProperty,
  setPropertyFromEntity,
  setPropertyFromProperty,
} from "./inferShared";
import _ from "lodash";

export function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const context = request.context;
  const entity = request.entity;

  setProperty({
    entity,
    toPath: ["metadata", "name"],
    value: `${entity.name}-service`,
  });

  // Do you have a k8s namespace? If so, set the namespace.
  setPropertyFromEntity({
    context,
    entityType: "k8sNamespace",
    fromPath: ["metadata", "name"],
    toEntity: entity,
    toPath: ["metadata", "namespace"],
  });

  setArrayEntryFromAllEntities({
    entity,
    context,
    entityType: "k8sDeployment",
    toPath: ["spec", "ports"],
    valuesCallback(
      fromEntity,
    ): ReturnType<SetArrayEntryFromAllEntities["valuesCallback"]> {
      const toSet: { path: string[]; value: any; system: string }[] = [];
      const containersBySystem: Record<
        string,
        Record<string, any>[]
      > = fromEntity.getPropertyForAllSystems({
        path: ["spec", "template", "spec", "containers"],
      });
      for (const system in containersBySystem) {
        const containers = containersBySystem[system];
        for (const container of containers) {
          if (container["ports"]) {
            for (const portDef of container["ports"]) {
              if (portDef["containerPort"]) {
                toSet.push({
                  path: ["port"],
                  value: portDef["containerPort"],
                  system,
                });
              }
              if (portDef["protocol"]) {
                toSet.push({
                  path: ["protocol"],
                  value: portDef["protocol"],
                  system,
                });
              }
            }
          }
        }
      }
      return toSet;
    },
  });

  setPropertyFromEntity({
    context,
    entityType: "k8sDeployment",
    fromPath: ["metadata", "labels", "app"],
    toEntity: entity,
    toPath: ["spec", "selector", "app"],
  });

  return { entity };
}

export default {
  inferProperties,
  checkQualifications: baseCheckQualifications,
  runCommands: baseRunCommands,
};
