import * as PIXI from "pixi.js";

import _ from "lodash";

import { Connector } from "./connector";

interface Position {
  x: number;
  y: number;
}

export enum SocketType {
  input = "input",
  output = "output",
}

export class Socket extends PIXI.Container {
  kind: string;
  type: SocketType;
  id: number;
  socketId: number;
  labelText: string | null;

  constructor(
    id: number,
    nodeId: number,
    labelText: string | null,
    type: SocketType,
    position: Position,
    color: number,
  ) {
    super();
    this.kind = "socket";

    this.id = id;

    this.name = `${nodeId}.${id}`;

    this.labelText = labelText;
    this.type = type;

    this.disableInteraction();
    this.setPosition(position);
    this.createConnector(this.name, type, color);

    if (labelText) {
      this.createLabel(labelText);
    }
  }

  createConnector(id: string, type: SocketType, color: number): void {
    const socket = new Connector(id, type, color);
    socket.name = "Connector";
    this.addChild(socket);
  }

  setConnected() {
    const connector = this.getChildByName("Connector");
    if (connector) connector.setConnected();
  }

  createLabel(text: string): void {
    const label = new PIXI.Text(text, {
      fontFamily: "Source Code Pro",
      fontSize: 9,
      fontWeight: "300",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    label.position.x = 10;
    label.position.y = -5;
    label.zIndex = 1;
    label.interactive = false;
    this.addChild(label);
  }

  setPosition(position: Position): void {
    this.position.x = position.x;
    this.position.y = position.y;
  }

  setZIndex(): void {
    this.zIndex = 2;
  }

  disableInteraction(): void {
    this.interactive = false;
  }
}
