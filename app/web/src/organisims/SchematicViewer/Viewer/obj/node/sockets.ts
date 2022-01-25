import * as PIXI from "pixi.js";

import * as MODEL from "../../../model";
import { Socket, SocketType } from "./sockets/socket";

const NODE_WIDTH = 140;

const INPUT_SOCKET_OFFSET = 45;
const OUTPUT_SOCKET_OFFSET = 35;
const SOCKET_SPACING = 20;

export class Sockets extends PIXI.Container {
  constructor(nodeId: string, inputs: MODEL.Socket[], outputs: MODEL.Socket[]) {
    super();

    this.setInputSockets(nodeId, inputs);
    this.setOutputSockets(nodeId, outputs);
  }

  setInputSockets(nodeId: string, inputs: MODEL.Socket[]): void {
    for (let i = 0; i < inputs.length; i++) {
      const s = inputs[i];

      let socketLabel = null;
      if (s.name) {
        socketLabel = s.name;
      }

      const socket = new Socket(
        s.id,
        nodeId,
        socketLabel,
        SocketType.input,
        {
          x: 0,
          y: INPUT_SOCKET_OFFSET + SOCKET_SPACING * i,
        },
        0xffdd44,
      );
      this.addChild(socket);
    }
  }

  setOutputSockets(nodeId: string, outputs: MODEL.Socket[]): void {
    for (let i = 0; i < outputs.length; i++) {
      const s = outputs[i];

      const socket = new Socket(
        s.id,
        nodeId,
        null,
        SocketType.output,
        {
          x: NODE_WIDTH,
          y: OUTPUT_SOCKET_OFFSET,
        },
        0xeb44ff,
      );
      this.addChild(socket);
    }
  }
}
