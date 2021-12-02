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
  kind: string;
  type: SocketType;
  id: string;
  labelText: string | null;

  constructor(
    id: string,
    labelText: string | null,
    type: SocketType,
    position: Position,
    color: number,
  ) {
    super();
    this.kind = "socket";

    this.id = id;
    this.labelText = labelText;
    this.type = type;

    this.beginFill(color);
    this.drawCircle(0, 0, 6);
    this.endFill();
    this.interactive = true;

    this.position.x = position.x;
    this.position.y = position.y;

    if (labelText) {
      const label = this.createLabel(labelText);
      this.addLabel(label);
    }

    this.name = id;
  }

  createLabel(text: string): PIXI.Text {
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
    label.zIndex = 2;
    label.interactive = false;
    return label;
  }

  addLabel(label: PIXI.Text): void {
    this.addChild(label);
  }
}
