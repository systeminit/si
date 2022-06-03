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

  constructor(id: string, type: SocketType, color: number) {
    super();

    this.id = id;
    this.type = type;
    this.kind = "socket";
    this.name = id;
    this.color = color;
    this.setDisconnected();
  }

  setConnected() {
    this.clear();
    this.beginFill(this.color);
    this.drawCircle(0, 0, 6);
    this.endFill();
  }

  setDisconnected() {
    this.clear();
    this.lineStyle(2, this.color);
    this.drawCircle(0, 0, 5.4);
    this.endFill();
    this.interactive = true;
  }
}
