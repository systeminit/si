import WebSocket from "ws";
import Debug from "debug";

import { Resource, SiEntity } from "si-entity";

import { SiCtx } from "../siCtx";
import intel from "../intel";
import { DecryptedSecret } from "../support";
import { tracer } from "../telemetry";
import api, { Span } from "@opentelemetry/api";
const debug = Debug("veritech:controllers:runCommand");

export interface RunCommandRequest {
  commandName: string;
  inputs: {
    name: string;
    args: Record<string, unknown>;
  };
  system: SiEntity;
  entity: SiEntity;
  resource: Resource;
  context: {
    entity: SiEntity;
    resource?: Resource;
    secret?: DecryptedSecret;
  }[];
}

export interface CommandProtocolStart {
  start: boolean;
}

export interface CommandProtocolOutputOutputLine {
  outputLine: string;
  errorLine?: never;
}

export interface CommandProtocolOutputErrorLine {
  outputLine?: never;
  errorLine: string;
}

export interface CommandProtocolOutput {
  output: CommandProtocolOutputOutputLine | CommandProtocolOutputErrorLine;
}

export interface CommandProtocolFinishSuccess {
  success: boolean;
  error?: never;
}

export interface CommandProtocolFinishError {
  success?: never;
  error: string;
}

export interface CommandProtocolFinish {
  finish: CommandProtocolFinishSuccess | CommandProtocolFinishError;
}

export type CommandProtocol =
  | CommandProtocolStart
  | CommandProtocolOutput
  | CommandProtocolFinish;

export type RunCommandCallback = (
  ctx: typeof SiCtx,
  request: RunCommandRequest,
  ws: WebSocket,
) => Promise<void>;

export interface RunCommandCallbacks {
  [commandName: string]: RunCommandCallback;
}

export async function runCommand(
  ws: WebSocket,
  req: string,
  parent: Span,
): Promise<void> {
  const ctx = api.trace.setSpan(api.context.active(), parent);
  const span = tracer.startSpan("runcommand.task", undefined, ctx);
  debug("/runCommand BEGIN");

  const request: RunCommandRequest = JSON.parse(req);
  request.entity = SiEntity.fromJson(request.entity);
  request.system = SiEntity.fromJson(request.system);
  for (const p of request.context) {
    p.entity = SiEntity.fromJson(p.entity);
  }
  debug("request %O", request);

  const entityType = request.entity.entityType;
  const intelFuncs = intel[entityType];
  if (
    intelFuncs &&
    intelFuncs.runCommands &&
    intelFuncs.runCommands[request.inputs.name]
  ) {
    send(ws, { start: true });
    try {
      await intelFuncs.runCommands[request.inputs.name](SiCtx, request, ws);
    } catch (e) {
      send(ws, { finish: { error: `command failed: ${e}` } });
    }
    send(ws, { finish: { success: true } });
  } else {
    send(ws, { finish: { error: "no command found" } });
  }
  close(ws);
  debug("finished");
  span.end();
}

function send(ws: WebSocket, message: CommandProtocol) {
  ws.send(JSON.stringify({ protocol: message }));
}

function close(ws: WebSocket) {
  ws.close(1000, "finished");
}
