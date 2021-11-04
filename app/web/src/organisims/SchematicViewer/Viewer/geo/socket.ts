import * as PIXI from "pixi.js";

import _ from "lodash";

interface Position {
  x: number;
  y: number;
}

export enum SocketType {
  input = "input",
  output = "output",
}

export class Socket extends PIXI.Graphics {
  name: string;
  kind: string;
  type: SocketType;
  id: string;

  constructor(
    name: string,
    type: SocketType,
    position: Position,
    color: number,
  ) {
    super();

    this.id = _.uniqueId();
    this.name = name;
    this.kind = "socket";
    this.type = type;

    // this.lineStyle(1.25, color);
    this.beginFill(color);
    this.drawCircle(0, 0, 6);
    this.endFill();
    this.interactive = true;

    this.position.x = position.x;
    this.position.y = position.y;
  }
}
