import * as PIXI from "pixi.js";

import _ from "lodash";

import { Connector } from "./connector";

export enum SocketType {
  input = "input",
  output = "output",
}

export class Socket extends PIXI.Container {
  kind: string;
  type: SocketType;
  id: number;
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
    this.addChild(socket);
  }

  isConnected() {
    for (const child of this.children) {
      if (child instanceof Connector) {
        return child.isConnected();
      }
    }
    return false;
  }

  setConnected() {
    for (const child of this.children) {
      if (child instanceof Connector) {
        child.setConnected();
      }
    }
  }

  setDisconnected() {
    for (const child of this.children) {
      if (child instanceof Connector) {
        child.setDisconnected();
      }
    }
  }

  createLabel(text: string): void {
    const label = new PIXI.Text(text, {
      fontFamily: "Inter",
      fontSize: 9,
      fontWeight: "300",
      letterSpacing: 1,
      fill: "white",
      align: "left",
    });
    label.position.x = 10;
    label.position.y = -7.5;
    label.zIndex = 1;
    label.interactive = false;
    label.resolution = window.devicePixelRatio * 2;
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

interface Position {
  x: number;
  y: number;
}
