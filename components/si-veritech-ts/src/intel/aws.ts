import { ResourceInternalHealth } from "si-entity";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import WebSocket from "ws";
import { awsAccessKeysEnvironment, awsRegion } from "../support";

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

  let awsEnv;
  try {
    awsEnv = awsAccessKeysEnvironment(req);
  } catch (e) {
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    response.state = "error";
    response.error = "Cannot find AWS access keys!";
    return response;
  }
  let region;
  try {
    region = awsRegion(req);
  } catch (e) {
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    response.state = "error";
    response.error = "Cannot determine AWS region!";
    return response;
  }
  const defaultArgs = ["--region", region];

  const output = await ctx.exec(
    "aws",
    [...defaultArgs, "iam", "list-access-keys"],
    {
      env: awsEnv,
      reject: false,
    },
  );

  if (output.exitCode != 0) {
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    response.state = "error";
    response.error = output.all;
  } else {
    response.health = "ok";
    response.internalHealth = ResourceInternalHealth.Ok;
    response.state = "ok";
  }
  return response;
}

export default { syncResource };
