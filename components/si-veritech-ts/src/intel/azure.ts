import { ResourceInternalHealth } from "si-entity";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import WebSocket from "ws";
import { azureLogin } from "../support";

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
  try {
    await azureLogin(req);
  } catch (e) {
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    response.state = "error";
    response.error = "Cannot authenticate with Azure";
    return response;
  }
  response.health = "ok";
  response.internalHealth = ResourceInternalHealth.Ok;
  response.state = "ok";
  return response;
}

export default { syncResource };
