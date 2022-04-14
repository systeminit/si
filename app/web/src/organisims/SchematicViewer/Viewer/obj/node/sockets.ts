import * as PIXI from "pixi.js";

import * as MODEL from "../../../model";
import { Socket, SocketType } from "./sockets/socket";
import { SchematicKind } from "@/api/sdf/dal/schematic";

const NODE_WIDTH = 140;
const NODE_HEIGHT = 100;

const INPUT_SOCKET_OFFSET = 45;
const OUTPUT_SOCKET_OFFSET = 35;
const SOCKET_SPACING = 20;

export class Sockets extends PIXI.Container {
  constructor(
    nodeId: number,
    inputs: MODEL.Socket[],
    outputs: MODEL.Socket[],
    panelKind: SchematicKind,
  ) {
    super();

    this.setInputSockets(nodeId, inputs, panelKind);
    this.setOutputSockets(nodeId, outputs, panelKind);
  }

  setInputSockets(
    nodeId: number,
    inputs: MODEL.Socket[],
    panelKind: SchematicKind,
  ): void {
    inputs = inputs.filter((i) => i.schematic_kind === panelKind);

    let pos, growth, color;
    let displaySocketLabel = false;
    switch (panelKind) {
      case SchematicKind.Component:
        pos = {
          x: 0,
          y: INPUT_SOCKET_OFFSET,
        };
        growth = { x: 0, y: SOCKET_SPACING };
        color = 0xffdd44;
        displaySocketLabel = true;
        break;
      case SchematicKind.Deployment:
        pos = {
          x: NODE_WIDTH / 2,
          y: 0,
        };
        growth = { x: SOCKET_SPACING, y: 0 };
        color = 0x80c037;
        displaySocketLabel = false;
        break;
    }

    for (let i = 0; i < inputs.length; i++) {
      const s = inputs[i];

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
        color,
      );
      this.addChild(socket);

      pos.x += growth.x;
      pos.y += growth.y;
    }
  }

  setOutputSockets(
    nodeId: number,
    outputs: MODEL.Socket[],
    panelKind: SchematicKind,
  ): void {
    outputs = outputs.filter((i) => i.schematic_kind === panelKind);

    let pos, color;
    switch (panelKind) {
      case SchematicKind.Component:
        pos = {
          x: NODE_WIDTH,
          y: OUTPUT_SOCKET_OFFSET,
        };
        color = 0xeb44ff;
        break;
      case SchematicKind.Deployment:
        pos = {
          x: NODE_WIDTH / 2,
          y: NODE_HEIGHT,
        };
        color = 0x00b0bc;
        break;
    }
    for (let i = 0; i < outputs.length; i++) {
      const s = outputs[i];

      const socket = new Socket(
        s.id,
        nodeId,
        null,
        SocketType.output,
        pos,
        color,
      );
      this.addChild(socket);
    }
  }
}
