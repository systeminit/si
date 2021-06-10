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

const debug = Debug("veritech:controllers:discover");

export interface DiscoveryRequest {
  entity: SiEntity;
  system: SiEntity;
  entityType: string;
  context: {
    entity: SiEntity;
    secret?: DecryptedSecret;
  }[];
}

export interface DiscoveryProtocolStart {
  start: boolean;
}

export interface DiscoverEntity {
  entity: SiEntity;
  configures: DiscoverEntity[];
}

export interface DiscoveryProtocolFinish {
  finish: {
    discovered: DiscoverEntity[];
    error?: string;
  };
}

export type DiscoveryCallback = (
  ctx: typeof SiCtx,
  request: DiscoveryRequest,
  ws: WebSocket,
) => Promise<DiscoveryProtocolFinish["finish"]>;

export type DiscoveryProtocol =
  | DiscoveryProtocolStart
  | DiscoveryProtocolFinish;

export async function discover(ws: WebSocket, req: string): Promise<void> {
  debug("/discover BEGIN");
  const request: DiscoveryRequest = JSON.parse(req);
  request.entity = SiEntity.fromJson(request.entity);
  request.system = SiEntity.fromJson(request.system);
  for (const p of request.context) {
    p.entity = SiEntity.fromJson(p.entity);
  }
  debug("request %O", request.entity.name);

  send(ws, { start: true });
  const intelFuncs = intel[request.entityType];
  if (intelFuncs && intelFuncs.discover) {
    try {
      const response = await intelFuncs.discover(SiCtx, request, ws);
      send(ws, { finish: response });
    } catch (e) {
      console.log("got here", e);
      const response: DiscoveryProtocolFinish["finish"] = {
        error: `discovery failed: ${e}`,
        discovered: [],
      };
      send(ws, { finish: response });
    }
  } else {
    console.log("nothing to see here");
    send(ws, { finish: { discovered: [] } });
  }
  close(ws);
  debug("finished");
}

function send(ws: WebSocket, message: DiscoveryProtocol) {
  ws.send(JSON.stringify({ protocol: message }));
}

function close(ws: WebSocket) {
  ws.close(1000, "finished");
}
