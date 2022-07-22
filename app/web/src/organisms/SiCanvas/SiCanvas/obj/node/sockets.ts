import * as PIXI from "pixi.js";

import { Socket, SocketType } from "./sockets/socket";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import {
  SchematicInputSocket,
  SchematicOutputSocket,
} from "@/api/sdf/dal/schematic";

const NODE_WIDTH = 160;
const NODE_HEIGHT = 120;

const INPUT_SOCKET_OFFSET = 60;
const OUTPUT_SOCKET_OFFSET = 60;
const SOCKET_SPACING = 20;

export class Sockets extends PIXI.Container {
  constructor(
    nodeId: number,
    inputs: SchematicInputSocket[],
    outputs: SchematicOutputSocket[],
    panelKind: SchematicKind,
  ) {
    super();

    this.setInputSockets(nodeId, inputs, panelKind);
    this.setOutputSockets(nodeId, outputs, panelKind);
  }

  setInputSockets(
    nodeId: number,
    inputs: SchematicInputSocket[],
    panelKind: SchematicKind,
  ): void {
    inputs = inputs.filter((i) => i.schematicKind === panelKind);

    let pos, growth;
    let displaySocketLabel = false;
    switch (panelKind) {
      case SchematicKind.Component:
        pos = {
          x: 0,
          y: INPUT_SOCKET_OFFSET,
        };
        growth = { x: 0, y: SOCKET_SPACING };
        displaySocketLabel = true;
        break;
      case SchematicKind.Deployment:
        pos = {
          x: NODE_WIDTH / 2,
          y: 0,
        };
        growth = { x: SOCKET_SPACING, y: 0 };
        displaySocketLabel = false;
        break;
    }

    for (const s of inputs) {
      let socketLabel = null;
      if (s.name && displaySocketLabel) {
        socketLabel = s.name;
      }

      const socket = new Socket(
        s.id,
        nodeId,
        socketLabel,
        SocketType.input,
        pos,
        s.provider.color,
      );
      this.addChild(socket);

      pos.x += growth.x;
      pos.y += growth.y;
    }
  }

  setOutputSockets(
    nodeId: number,
    outputs: SchematicOutputSocket[],
    panelKind: SchematicKind,
  ): void {
    outputs = outputs.filter((i) => i.schematicKind === panelKind);

    let pos;
    switch (panelKind) {
      case SchematicKind.Component:
        pos = {
          x: NODE_WIDTH,
          y: OUTPUT_SOCKET_OFFSET,
        };
        break;
      case SchematicKind.Deployment:
        pos = {
          x: NODE_WIDTH / 2,
          y: NODE_HEIGHT,
        };
        break;
    }
    for (const s of outputs) {
      const socket = new Socket(
        s.id,
        nodeId,
        null,
        SocketType.output,
        pos,
        s.provider.color,
      );
      this.addChild(socket);
    }
  }
}
