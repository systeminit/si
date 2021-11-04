import * as PIXI from "pixi.js";

interface Position {
  x: number;
  y: number;
}

export interface PanningPositionData {
  current: {
    x: number;
    y: number;
  };
}

export class PanningManager {
  data?: PIXI.InteractionData | undefined;
  position?: PanningPositionData | undefined;
  offset?: Position | undefined;

  constructor() {}

  beforePan(data: PIXI.InteractionData, offset: Position): void {
    this.data = data;
    this.position = {
      current: {
        x: data.global.x,
        y: data.global.y,
      },
    };
    this.offset = offset;
  }

  pan(data: PIXI.InteractionData, container: PIXI.Container): void {
    const newPosition = {
      x: data.global.x,
      y: data.global.y,
    };

    if (this.position && this.offset) {
      const transform = {
        x: newPosition.x - this.offset.x, // - this.position.current.x,
        y: newPosition.y - this.offset.y, //- this.position.current.y,
      };

      // limit panning within a predefined canvas size
      container.x = transform.x;
      container.y = transform.y;
      container.updateTransform();

      this.position.current.x = container.x;
      this.position.current.y = container.y;
    }
  }
}
