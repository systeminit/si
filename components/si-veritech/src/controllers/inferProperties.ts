import { Context } from "koa";

import Debug from "debug";
const debug = Debug("veritech:controllers:inferProperties");

import { SiEntity as Entity, Resource } from "si-entity";
import { registry } from "si-registry";

import intel from "../intel";

export interface InferPropertiesRequest {
  entityType: string;
  entity: Entity;
  resources: Resource[];
  predecessors: {
    entity: Entity;
    resources: Resource[];
  }[];
}

export interface InferPropertiesReply {
  entity: Entity;
}

export interface InferPropertiesResult {
  entity: Entity;
}

export function inferProperties(ctx: Context): void {
  debug("/inferProperties BEGIN");
  debug("request body: %O", ctx.request.body);
  const request: InferPropertiesRequest = ctx.request.body;
  const registryObj = registry[request.entityType];
  if (!registryObj) {
    ctx.response.status = 400;
    ctx.response.body = {
      code: 400,
      message: `Cannot find registry entry for ${request.entityType}`,
    };
    return;
  }

  // Check if this object has the right intel functions
  if (intel[request.entityType] && intel[request.entityType].inferProperties) {
    request.entity = Entity.fromJson(request.entity);
    const result = intel[request.entityType].inferProperties(request);
    result.entity.computeProperties();
    debug("response body: %O", result);
    debug("/inferProperties END");
    ctx.response.body = result;
  } else {
    debug("default response");
    debug("/inferProperties END");
    request.entity = Entity.fromJson(request.entity);
    request.entity.computeProperties();
    ctx.response.status = 200;
    ctx.response.body = { entity: request.entity };
  }
}
