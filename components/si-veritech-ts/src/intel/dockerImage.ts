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
import { findSecret, SecretKind, skopeoDockerHubAuth } from "../support";
import path from "path";

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
  async dockerImageExistsInRegistry(ctx, q, req) {
    const image = req.entity.getProperty({
      system: req.system.id,
      path: ["image"],
    }) as string;
    const args = ["inspect", "--config"];
    let authTempDir;
    const secret = findSecret(req, SecretKind.DockerHub);
    if (secret) {
      authTempDir = await skopeoDockerHubAuth({
        username: secret.username ? secret.username : "<username-not-set>",
        password: secret.password ? secret.password : "<password-not-set>",
      });

      args.push("--authfile");
      args.push(path.join(authTempDir.path, "auth.json"));
    }
    const skopeoInspect = await ctx.exec("skopeo", [
      ...args,
      `docker://docker.io/${image}`,
    ]);
    return {
      name: q.name,
      qualified: !skopeoInspect.failed,
      output: skopeoInspect.all,
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
  const image = req.entity.getProperty({
    system,
    path: ["image"],
  });
  const args = ["inspect", "--config"];
  let authTempDir;
  const secret = findSecret(req, SecretKind.DockerHub);
  if (secret) {
    authTempDir = await skopeoDockerHubAuth({
      username: secret.username ? secret.username : "<username-not-set>",
      password: secret.password ? secret.password : "<password-not-set>",
    });

    args.push("--authfile");
    args.push(path.join(authTempDir.path, "auth.json"));
  }
  const result = await ctx.exec("skopeo", [
    ...args,
    `docker://docker.io/${image}`,
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
    response.data["inspect"] = JSON.parse(result.stdout);
  }
  return response;
}

export default {
  inferProperties,
  checkQualifications,
  runCommands,
  syncResource,
};
