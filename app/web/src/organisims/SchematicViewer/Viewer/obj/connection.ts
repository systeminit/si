import * as PIXI from "pixi.js";

import _ from "lodash";

export enum ConnectionType {
  interactive = "interactive",
  standard = "standard",
}

interface Point2D {
  x: number;
  y: number;
}

export class Connection extends PIXI.Graphics {
  id: string;
  kind: string;
  type: ConnectionType;
  sourceSocketId: string;
  destinationSocketId: string;

  constructor(
    source: Point2D,
    destination: Point2D,
    sourceSocketId: string,
    destinationSocketId: string,
    _interactive?: boolean,
  ) {
    super();
    this.id = _.uniqueId();
    this.kind = "connection";

    if (_interactive) {
      this.type = ConnectionType.interactive;
    } else {
      this.type = ConnectionType.standard;
    }

    this.name = "src:" + sourceSocketId + "-" + "dest:" + destinationSocketId;
    this.sourceSocketId = sourceSocketId;
    this.destinationSocketId = destinationSocketId;

    this.initiailize(source, destination);
  }

  private initiailize(p1: Point2D, p2: Point2D) {
    this.lineStyle(1.25, 0xb0b0b0);
    this.moveTo(0, 0);
    this.lineTo(p2.x - p1.x, p2.y - p1.y);
    this.position.x = p1.x;
    this.position.y = p1.y;
  }

  update(p1: Point2D, p2: Point2D) {
    this.clear();
    this.lineStyle(1.25, 0xb0b0b0);

    this.moveTo(0, 0);
    this.lineTo(p2.x - p1.x, p2.y - p1.y);
    this.position.x = p1.x;
    this.position.y = p1.y;
  }
}
