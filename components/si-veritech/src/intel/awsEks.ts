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
  _ctx: typeof SiCtx,
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

  const cluster = findEntityByType(req, "awsEksCluster");

  if (!cluster) {
    response.error = "No cluster connected";
    response.state = "error";
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    return response;
  }
  const clusterResource = _.find(
    req.resourceContext,
    (r) => r.entityId == cluster.id,
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
  return response;
}

export default { syncResource };
