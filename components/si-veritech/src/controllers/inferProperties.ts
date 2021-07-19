import { Context } from "koa";
import api from "@opentelemetry/api";

import Debug from "debug";
const debug = Debug("veritech:controllers:inferProperties");

import { SiEntity as Entity, Resource, OpType, OpSource } from "si-entity";
import { registry, RegistryEntry } from "si-registry";

import intel from "../intel";

export interface InferPropertiesRequest {
  entityType: string;
  entity: Entity;
  context: Entity[];
}

export interface InferPropertiesReply {
  entity: Entity;
}

export interface InferPropertiesResult {
  entity: Entity;
}

export function inferProperties(ctx: Context): void {
  const span = api.trace.getSpan(api.context.active());
  span.updateName("veritech.inferproperties");
  debug("request body: %O", ctx.request.body);
  const request: InferPropertiesRequest = ctx.request.body;
  span.setAttributes({
    "si.entity.type": request.entityType,
    "si.entity.id": request.entity.id,
  });

  const registryObj = registry[request.entityType];
  if (!registryObj) {
    ctx.response.status = 400;
    ctx.response.body = {
      code: 400,
      message: `Cannot find registry entry for ${request.entityType}`,
    };
    return;
  }

  request.entity = Entity.fromJson(request.entity);
  request.entity.setDefaultProperties();
  for (let x = 0; x < request.context.length; x++) {
    request.context[x] = Entity.fromJson(request.context[x]);
  }

  // Check if this object has the right intel functions
  if (intel[request.entityType] && intel[request.entityType].inferProperties) {
    const result = intel[request.entityType].inferProperties(request);
    result.entity.computeProperties();
    debug("response body: %O", result);
    ctx.response.body = result;
  } else {
    debug("default response");
    request.entity.computeProperties();
    ctx.response.status = 200;
    ctx.response.body = { entity: request.entity };
  }
}
