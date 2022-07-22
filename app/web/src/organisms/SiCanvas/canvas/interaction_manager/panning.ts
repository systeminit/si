import * as PIXI from "pixi.js";

interface PanningState {
  data: PIXI.InteractionData;
  offset: Position;
}

export class PanningManager {
  state?: PanningState;

  beforePan(data: PIXI.InteractionData, offset: Position): void {
    this.state = {
      data,
      offset,
    };
  }

  pan(data: PIXI.InteractionData, container: PIXI.Container): void {
    if (this.state) {
      const transform = {
        x: data.global.x - this.state.offset.x,
        y: data.global.y - this.state.offset.y,
      };

      // limit panning within a predefined canvas size
      container.x = transform.x;
      container.y = transform.y;
      container.updateTransform();
    }
  }
}

interface Position {
  x: number;
  y: number;
}
