import { ResourceInternalHealth } from "si-entity";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import WebSocket from "ws";
import _ from "lodash";
import { findEntityByType } from "../support";

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

  const implementation = req.entity.getProperty({
    system,
    path: ["implementation"],
  });
  if (!implementation) {
    response.error = "No implementation selected";
    response.state = "error";
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    return response;
  }
  const implementationResource = _.find(
    req.resourceContext,
    (r) => r.entityId == implementation,
  );
  if (implementationResource) {
    response.data = implementationResource.data;
    response.error = implementationResource.error;
    response.state = implementationResource.state;
    response.health = implementationResource.health;
    response.internalHealth = implementationResource.internalHealth;
    response.internalStatus = implementationResource.internalStatus;
    response.subResources = implementationResource.subResources;
  } else {
    response.state = "unknown";
    response.health = "unknown";
    response.internalHealth = ResourceInternalHealth.Unknown;
    return response;
  }

  const healthChecks: any = req.entity.getProperty({
    system,
    path: ["healthChecks"],
  });
  if (healthChecks) {
    console.log("healthChecks");
    for (const healthCheck of healthChecks) {
      console.log("healthChecks", { healthCheck });
      if (healthCheck["protocol"] == "HTTP") {
        const host = healthCheck["host"];
        const port = healthCheck["port"];
        const path = healthCheck["path"];
        if (host) {
          let url = `http://${host}`;
          if (port) {
            url += `:${port}`;
          }
          if (path) {
            url += path;
          }
          // @ts-ignore
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

export default { syncResource };
