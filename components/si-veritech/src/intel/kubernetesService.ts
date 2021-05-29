import { ResourceInternalHealth } from "si-entity";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import WebSocket from "ws";
import _ from "lodash";
import { findEntityByType } from "../support";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import {
  SetArrayEntryFromAllEntities,
  setArrayEntryFromAllEntities,
} from "./inferShared";

export function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const context = request.context;
  const entity = request.entity;

  setArrayEntryFromAllEntities({
    entity,
    context,
    entityType: "k8sService",
    toPath: ["healthChecks"],
    valuesCallback(
      fromEntity,
    ): ReturnType<SetArrayEntryFromAllEntities["valuesCallback"]> {
      const toSet: { path: string[]; value: any; system: string }[] = [];

      const portsBySystem: Record<
        string,
        Record<string, any>[]
      > = fromEntity.getPropertyForAllSystems({
        path: ["spec", "ports"],
      });
      for (const system in portsBySystem) {
        const ports = portsBySystem[system];
        for (const port of ports) {
          if (port["port"] == "80" || port["port"] == "8080") {
            toSet.push({
              path: ["protocol"],
              value: "HTTP",
              system,
            });
            toSet.push({
              path: ["port"],
              value: port["port"],
              system,
            });
          }
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
  const system = req.system.id;
  const response: CommandProtocolFinish["finish"] = {
    data: {},
    state: req.resource.state,
    health: req.resource.health,
    internalStatus: req.resource.internalStatus,
    internalHealth: req.resource.internalHealth,
    subResources: req.resource.subResources,
  };

  const service = findEntityByType(req, "k8sService");

  if (!service) {
    response.error = "No service connected";
    response.state = "error";
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    return response;
  }
  const clusterResource = _.find(
    req.resourceContext,
    (r) => r.entityId == service.id,
  );
  if (clusterResource) {
    response.data = clusterResource.data;
    response.error = clusterResource.error;
    response.state = clusterResource.state;
    response.health = clusterResource.health;
    response.internalHealth = clusterResource.internalHealth;
    response.internalStatus = clusterResource.internalStatus;
    response.subResources = clusterResource.subResources;
  } else {
    response.state = "unknown";
    response.health = "unknown";
    response.internalHealth = ResourceInternalHealth.Unknown;
    return response;
  }

  const healthChecks = req.entity.getProperty({
    system,
    path: ["healthChecks"],
  });
  if (healthChecks) {
    // @ts-ignore
    for (const healthCheck of healthChecks) {
      if (healthCheck["protocol"] == "HTTP") {
        const host = healthCheck["host"];
        const port = healthCheck["port"];
        const path = healthCheck["path"];
        if (!host) {
          for (const kubeClusterId of Object.keys(response.subResources)) {
            if (
              response.data[kubeClusterId] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"]["status"] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"]["status"]["loadBalancer"] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"]["status"]["loadBalancer"][
                "ingress"
              ]
            ) {
              // @ts-ignore
              for (const ingress of response.data[kubeClusterId]["data"][
                "status"
              ]["loadBalancer"]["ingress"]) {
                let hostName;
                // @ts-ignore
                if (ingress["hostname"]) {
                  hostName = ingress["hostname"];
                } else {
                  hostName = ingress["ip"];
                }
                if (hostName) {
                  let url = `http://${hostName}:${port}`;
                  if (path) {
                    url += path;
                  }
                  response.subResources[kubeClusterId].data[
                    "healthCheckUrl"
                  ] = url;
                  const result = await ctx.exec(
                    "curl",
                    ["-v", "-i", "-m", "10", url],
                    {
                      reject: false,
                    },
                  );
                  if (result.exitCode != 0) {
                    response.health = "error";
                    response.state = "error";
                    response.internalHealth = ResourceInternalHealth.Error;
                    response.error = result.all;
                    if (response.subResources[kubeClusterId]) {
                      response.subResources[kubeClusterId].health = "error";
                      response.subResources[kubeClusterId].state = "error";
                      response.subResources[kubeClusterId].internalHealth =
                        ResourceInternalHealth.Error;
                      response.subResources[kubeClusterId].error = result.all;
                    }
                  }
                  response.data[kubeClusterId] =
                    response.subResources[kubeClusterId];
                }
              }
            }
          }
        } else {
          let url = `http://${host}:${port}`;
          if (path) {
            url += path;
          }
          // @ts-ignore
          response.subResources[kubeClusterId]["healthCheckUrl"] = url;
          response.data["healthCheckUrl"] = url;
          const result = await ctx.exec("curl", ["-v", "-i", "-m", "10", url], {
            reject: false,
          });
          if (result.exitCode != 0) {
            response.health = "error";
            response.state = "error";
            response.internalHealth = ResourceInternalHealth.Error;
            response.error = result.all;
          }
        }
      }
    }
  }
  return response;
}

export default { inferProperties, syncResource };
