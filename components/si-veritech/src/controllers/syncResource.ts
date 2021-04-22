import WebSocket from "ws";
import Debug from "debug";

import { SiEntity, Resource, ResourceStatus, ResourceHealth } from "si-entity";

const debug = Debug("veritech:controllers:syncResource");

export interface SyncResourceRequest {
  entity: SiEntity;
  resource: Resource;
  predecessors: {
    entity: SiEntity;
    resource: Resource;
  }[];
}

export interface CommandProtocolStart {
  start: boolean;
}

export interface CommandProtocolFinish {
  finish: {
    state: Record<string, unknown>;
    status: ResourceStatus;
    health: ResourceHealth;
    error?: string;
  };
}

export type CommandProtocol = CommandProtocolStart | CommandProtocolFinish;

export async function syncResource(ws: WebSocket, req: string): Promise<void> {
  debug("/syncResource BEGIN");
  const request: SyncResourceRequest = JSON.parse(req);
  debug("request %O", request);

  send(ws, { start: true });
  send(ws, {
    finish: {
      state: request.resource.state,
      status: request.resource.status,
      health: request.resource.health,
      error: "uh no, doing nothing",
    },
  });
  close(ws);
  debug("finished");
}

function send(ws: WebSocket, message: CommandProtocol) {
  ws.send(JSON.stringify({ protocol: message }));
}

function close(ws: WebSocket) {
  ws.close(1000, "finished");
}
