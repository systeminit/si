import { OpSource } from "si-entity/dist/siEntity";
import { Qualification } from "si-registry";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import Debug from "debug";
const debug = Debug("veritech:controllers:intel:dockerImage");
import {
  CheckQualificationsItem,
  CheckQualificationsRequest,
} from "../controllers/checkQualifications";
import { SiCtx } from "../siCtx";

import { RunCommandCallbacks } from "../controllers/runCommand";
import {
  CommandProtocolFinish,
  SyncResourceRequest,
} from "../controllers/syncResource";
import WebSocket from "ws";
import { ResourceInternalHealth } from "si-entity";

function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const entity = request.entity;

  const reply = entity.set({
    source: OpSource.Inferred,
    system: "baseline",
    path: ["image"],
    value: request.entity.name,
  });
  debug("failed to set", { reply });

  return { entity: request.entity };
}

export type CheckQualificationCallback = (
  ctx: typeof SiCtx,
  qualification: Qualification,
  request: CheckQualificationsRequest,
) => Promise<CheckQualificationsItem>;

export interface CheckQualificationCallbacks {
  [qualificationName: string]: CheckQualificationCallback;
}

export const checkQualifications: CheckQualificationCallbacks = {
  async dockerImageExistsInRegistry(ctx, q, r) {
    const image = r.entity.getProperty({
      system: r.systemId,
      path: ["image"],
    }) as string;
    const dockerPull = await ctx.exec("docker", ["pull", image]);
    return {
      name: q.name,
      qualified: !dockerPull.failed,
      output: dockerPull.all,
    };
  },
};

export const runCommands: RunCommandCallbacks = {
  "universal:deploy": async function (ctx, req, ws) {
    await ctx.execStream(ws, "docker", [
      "pull",
      req.entity.getProperty({
        system: req.system.id,
        path: ["image"],
      }),
    ]);
  },
};

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
  const result = await ctx.exec("docker", [
    "pull",
    req.entity.getProperty({
      system,
      path: ["image"],
    }),
  ]);
  if (result.exitCode != 0) {
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    response.error = result.all;
    response.state = "error";
  } else {
    response.health = "ok";
    response.internalHealth = ResourceInternalHealth.Ok;
    response.state = "ok";
    const inspectResult = await ctx.exec("docker", [
      "image",
      "inspect",
      req.entity.getProperty({
        system,
        path: ["image"],
      }),
    ]);
    if (inspectResult.exitCode != 0) {
      response.error = inspectResult.all;
    } else {
      response.data["inspect"] = JSON.parse(inspectResult.stdout);
    }
  }
  return response;
}

export default {
  inferProperties,
  checkQualifications,
  runCommands,
  syncResource,
};
