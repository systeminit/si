import WebSocket from "ws";
import Debug from "debug";

import {
  SiEntity,
  SubResource,
  Resource,
  ResourceInternalStatus,
  ResourceInternalHealth,
} from "si-entity";
import { DecryptedSecret } from "../support";
import intel from "../intel";
import { SiCtx } from "../siCtx";

const debug = Debug("veritech:controllers:syncResource");

export interface SyncResourceRequest {
  entity: SiEntity;
  resource: Resource;
  system: SiEntity;
  context: {
    entity: SiEntity;
    secret?: DecryptedSecret;
  }[];
  resourceContext: Resource[];
}

export interface CommandProtocolStart {
  start: boolean;
}

export interface CommandProtocolFinish {
  finish: {
    data: Record<string, unknown>;
    state: string;
    health: string;
    internalStatus: ResourceInternalStatus;
    internalHealth: ResourceInternalHealth;
    subResources: Record<string, SubResource>;
    error?: string;
  };
}

export type SyncResourceCallback = (
  ctx: typeof SiCtx,
  request: SyncResourceRequest,
  ws: WebSocket,
) => Promise<CommandProtocolFinish["finish"]>;

export type CommandProtocol = CommandProtocolStart | CommandProtocolFinish;

// TODO: Plumb the callback through, and then implement the actual resource data
// for given things.
export async function syncResource(ws: WebSocket, req: string): Promise<void> {
  debug("/syncResource BEGIN");
  const request: SyncResourceRequest = JSON.parse(req);
  request.entity = SiEntity.fromJson(request.entity);
  request.system = SiEntity.fromJson(request.system);
  for (const p of request.context) {
    p.entity = SiEntity.fromJson(p.entity);
  }
  debug("request %O", request);

  send(ws, { start: true });
  const entityType = request.entity.entityType;
  const intelFuncs = intel[entityType];
  if (intelFuncs && intelFuncs.syncResource) {
    try {
      const response = await intelFuncs.syncResource(SiCtx, request, ws);
      send(ws, { finish: response });
    } catch (e) {
      const response = {
        data: request.resource.data,
        state: request.resource.state,
        health: request.resource.health,
        internalStatus: request.resource.internalStatus,
        internalHealth: request.resource.internalHealth,
        subResources: request.resource.subResources,
        error: `sync failed: ${e}`,
      };
      send(ws, { finish: response });
    }
  } else {
    const response = {
      data: request.resource.data,
      state: "created",
      health: "ok",
      internalStatus: ResourceInternalStatus.Created,
      internalHealth: ResourceInternalHealth.Ok,
      subResources: request.resource.subResources,
    };
    send(ws, { finish: response });
  }
  close(ws);
  debug("finished");
}

function send(ws: WebSocket, message: CommandProtocol) {
  ws.send(JSON.stringify({ protocol: message }));
}

function close(ws: WebSocket) {
  ws.close(1000, "finished");
}
