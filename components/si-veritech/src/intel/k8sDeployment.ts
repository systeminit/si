import { OpSource, SiEntity } from "si-entity/dist/siEntity";
import Debug from "debug";
const debug = Debug("veritech:controllers:intel:k8sDeployment");
import {
  baseInferProperties,
  baseCheckQualifications,
  baseRunCommands,
  baseSyncResource,
  forEachCluster,
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
import { RunCommandCallbacks } from "../controllers/runCommand";
import { findEntityByType } from "../support";
import {
  CommandProtocolFinish,
  SyncResourceRequest,
} from "../controllers/syncResource";
import WebSocket from "ws";
import { ResourceInternalHealth } from "si-entity";
import { SiCtx } from "../siCtx";

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

  setPropertyFromProperty({
    entity,
    fromPath: ["metadata", "labels", "app"],
    toPath: ["spec", "selector", "matchLabels", "app"],
  });

  setPropertyFromProperty({
    entity,
    fromPath: ["metadata", "labels", "app"],
    toPath: ["spec", "template", "metadata", "labels", "app"],
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
    toPath: ["spec", "template", "metadata", "namespace"],
  });

  setArrayEntryFromAllEntities({
    entity,
    context,
    entityType: "dockerImage",
    toPath: ["spec", "template", "spec", "containers"],
    valuesCallback(
      fromEntity,
    ): ReturnType<SetArrayEntryFromAllEntities["valuesCallback"]> {
      const toSet: { path: string[]; value: any; system: string }[] = [];
      toSet.push({
        path: ["name"],
        value: fromEntity.name,
        system: "baseline",
      });
      const imageValues = fromEntity.getPropertyForAllSystems({
        path: ["image"],
      });
      for (const system in imageValues) {
        toSet.push({ path: ["image"], value: imageValues[system], system });
      }
      const exposedPortValues = fromEntity.getPropertyForAllSystems({
        path: ["ExposedPorts"],
      });
      for (const system in exposedPortValues) {
        const exposedPortList: string[] = exposedPortValues[system] as string[];
        for (const exposedPortValue of exposedPortList) {
          const exposedPortParts: string[] = exposedPortValue.split("/");
          const portNumber = exposedPortParts[0];
          const portProtocol = exposedPortParts[1]
            ? exposedPortParts[1].toUpperCase()
            : "TCP";
          toSet.push({
            path: ["ports"],
            value: { containerPort: portNumber, protocol: portProtocol },
            system,
          });
        }
      }
      return toSet;
    },
  });

  return { entity };
}

export async function syncResource(
  ctx: typeof SiCtx,
  req: SyncResourceRequest,
  ws: WebSocket,
): Promise<CommandProtocolFinish["finish"]> {
  const response = await baseSyncResource(ctx, req, ws);
  const nameSpace = findEntityByType(req, "k8sNamespace");
  const system = req.system.id;
  const defaultArgs = ["rollout", "status", "deployment", "-w", "--timeout=5s"];
  if (nameSpace) {
    defaultArgs.push("-n");
    defaultArgs.push(
      nameSpace.getProperty({ system, path: ["metadata", "name"] }),
    );
  }

  await forEachCluster(
    ctx,
    req,
    ws,
    async (_kubeYaml, kubeConfigDir, kubeCluster) => {
      const result = await ctx.exec(
        "kubectl",
        [
          ...defaultArgs,
          "--kubeconfig",
          `${kubeConfigDir.path}/config`,
          req.entity.getProperty({ system, path: ["metadata", "name"] }),
        ],
        { reject: false },
      );
      if (result.exitCode != 0) {
        response.health = "error";
        response.state = "error";
        response.internalHealth = ResourceInternalHealth.Error;
        response.error = result.all;
        if (response.subResources[kubeCluster.id]) {
          // @ts-ignore
          response.subResources[kubeCluster.id].state = "error";
          // @ts-ignore
          response.subResources[kubeCluster.id].health = "error";
          // @ts-ignore
          response.subResources[kubeCluster.id].internalHealth =
            ResourceInternalHealth.Error;
          // @ts-ignore
          response.subResources[kubeCluster.id].error = result.all;
        }
      }
    },
  );
  response.data = response.subResources;
  return response;
}

export default {
  inferProperties,
  checkQualifications: baseCheckQualifications,
  runCommands: baseRunCommands,
  syncResource,
};
