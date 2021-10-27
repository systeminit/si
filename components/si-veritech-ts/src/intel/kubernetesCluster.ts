import { ResourceInternalHealth } from "si-entity";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import WebSocket from "ws";
import _ from "lodash";

export async function syncResource(
  ctx: typeof SiCtx,
  req: SyncResourceRequest,
  _ws: WebSocket,
): Promise<CommandProtocolFinish["finish"]> {
  const response: CommandProtocolFinish["finish"] = {
    data: {},
    state: req.resource.state,
    health: req.resource.health,
    internalStatus: req.resource.internalStatus,
    internalHealth: req.resource.internalHealth,
    subResources: req.resource.subResources,
  };

  const system = req.system.id;
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
  return response;
}

export default { syncResource };
