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
    debug("hello from inside");
    await ctx.execStream(ws, "docker", [
      "pull",
      req.entity.getProperty({
        system: req.system.id,
        path: ["image"],
      }),
    ]);
  },
};

export default { inferProperties, checkQualifications, runCommands };
