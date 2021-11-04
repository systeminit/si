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
  name: string;
  kind: string;
  type: ConnectionType;
  sourceSocketId: string;
  destinationSocketId: string;

  constructor(
    p1: Point2D,
    p2: Point2D,
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

    this.initiailize(p1, p2);
  }

  private initiailize(p1: Point2D, p2: Point2D) {
    const ctp1 = {
      x: 0,
      y: 0,
    };
    const ctp2 = {
      x: 0,
      y: 0,
    };

    this.lineStyle(1.25, 0xb0b0b0);
    // this.bezierCurveTo(
    //   ctp1.x,
    //   ctp1.y,
    //   ctp2.x,
    //   ctp2.y,
    //   p2.x - p1.x,
    //   p2.y - p1.y,
    // );

    this.moveTo(0, 0);
    this.lineTo(p2.x - p1.x, p2.y - p1.y);
    this.position.x = p1.x;
    this.position.y = p1.y;
  }

  update(p1: Point2D, p2: Point2D) {
    const ctp1 = {
      x: 0,
      y: 0,
    };
    const ctp2 = {
      x: 0,
      y: 0,
    };

    this.clear();
    this.lineStyle(1.25, 0xb0b0b0);
    // this.bezierCurveTo(
    //   ctp1.x,
    //   ctp1.y,
    //   ctp2.x,
    //   ctp2.y,
    //   p2.x - p1.x,
    //   p2.y - p1.y,
    // );
    this.moveTo(0, 0);
    this.lineTo(p2.x - p1.x, p2.y - p1.y);
    this.position.x = p1.x;
    this.position.y = p1.y;
  }
}


// Fix interactive conn when not connecting!