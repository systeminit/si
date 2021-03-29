import { OpSource } from "si-entity/dist/siEntity";
import { Qualification } from "si-registry";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import Debug from "debug";
const debug = Debug("veritech:controllers:inferProperties:dockerImage");
import {
  CheckQualificationsItem,
  CheckQualificationsRequest,
} from "../controllers/checkQualifications";
import { SiCtx } from "../siCtx";

import _ from "lodash";

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
    debug("entity name", {
      image,
      systemId: r.systemId,
      properties: r.entity.properties,
    });
    const dockerPull = await ctx.exec("docker", ["pull", image]);
    return {
      name: q.name,
      qualified: !dockerPull.failed,
      output: dockerPull.all,
    };
  },
  async dockerImageIsTrue(ctx, q, r) {
    return {
      name: q.name,
      qualified: true,
      output: `be my fate: ${Date.now()}`,
    };
  },
};

export default { inferProperties, checkQualifications };
