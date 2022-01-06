import * as PIXI from "pixi.js";

export enum SocketType {
  input = "input",
  output = "output",
}

export class SocketConnector extends PIXI.Graphics {
  id: string;
  type: SocketType;
  kind: string;

  constructor(id: string, type: SocketType, color: number) {
    super();

    this.id = id;
    this.type = type;
    this.kind = "socket";
    this.name = id;

    this.beginFill(color);
    this.drawCircle(0, 0, 6);
    this.endFill();
    this.interactive = true;
  }
}
