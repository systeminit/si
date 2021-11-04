import * as PIXI from "pixi.js";

import { Node } from "../geo";
import { translateNode } from "../transform";

interface Position {
  x: number;
  y: number;
}

export interface DraggingInteractionData {
  position: {
    mouse: {
      x: number;
      y: number;
    };
  };
}

export class DraggingManager {
  data?: PIXI.InteractionData | undefined;
  offset?: Position | undefined;

  constructor() {}

  beforeDrag(data: PIXI.InteractionData, offset: Position): void {
    this.data = data;
    this.offset = offset;
  }

  drag(node: Node): void {
    if (this.data && this.offset) {
      const localPosition = this.data.getLocalPosition(node.parent);
      const position = {
        x: localPosition.x - this.offset.x,
        y: localPosition.y - this.offset.y,
      };
      translateNode(node, position);
    }
  }

  afterDrag(): void {}
}
