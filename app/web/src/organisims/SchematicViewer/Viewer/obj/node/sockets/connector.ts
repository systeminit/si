import * as PIXI from "pixi.js";

export enum SocketType {
  input = "input",
  output = "output",
}

export class Connector extends PIXI.Graphics {
  id: string;
  type: SocketType;
  kind: string;
  color: number;
  connected: boolean;

  constructor(id: string, type: SocketType, color: number) {
    super();

    this.id = id;
    this.type = type;
    this.kind = "socket";
    this.name = id;
    this.color = color;
    this.connected = false;
    this.setDisconnected();
  }

  isConnected() {
    return this.connected;
  }

  setConnected() {
    this.clear();
    this.beginFill(this.color);
    this.drawCircle(0, 0, 6);
    this.endFill();
    this.connected = true;
  }

  setDisconnected() {
    this.clear();

    this.beginFill(0x282e30);
    this.drawCircle(0, 0, 6);
    this.endFill();

    this.lineStyle(1, this.color);
    this.drawCircle(0, 0, 5.5);
    this.endFill();
    this.interactive = true;
    this.connected = false;
  }
}
