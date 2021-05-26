import { OpSource } from "si-entity/dist/siEntity";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import { setPropertyFromEntity } from "./inferShared";
import { SiCtx } from "../siCtx";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { ResourceInternalHealth } from "si-entity";
import { azureLogin } from "../support";
import WebSocket from "ws";

function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const entity = request.entity;
  const context = request.context;

  entity.set({
    source: OpSource.Inferred,
    system: "baseline",
    path: ["name"],
    value: request.entity.name,
  });

  setPropertyFromEntity({
    context,
    entityType: "awsLocation",
    fromPath: ["location"],
    toEntity: entity,
    toPath: ["location"],
  });

  return { entity: request.entity };
}

export async function syncResource(
  ctx: typeof SiCtx,
  req: SyncResourceRequest,
  _ws: WebSocket,
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
  await azureLogin(req);
  const name: string | null = req.entity.getProperty({
    system,
    path: ["name"],
  });
  if (name) {
    const result = await ctx.exec("az", [
      "group",
      "show",
      "--resource-group",
      name,
    ]);
    response.data = JSON.parse(result.stdout);
    if (result.exitCode != 0) {
      response.health = "error";
      response.internalHealth = ResourceInternalHealth.Error;
      response.state = "error";
    } else {
      response.health = "ok";
      response.internalHealth = ResourceInternalHealth.Ok;
      response.state = "ok";
    }
  }

  return response;
}

export default { inferProperties, syncResource };
