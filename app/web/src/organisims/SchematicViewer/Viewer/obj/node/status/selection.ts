import * as PIXI from "pixi.js";

export class SelectionStatus extends PIXI.Container {
  constructor(width: number, height: number, radius: number) {
    super();

    const status = new PIXI.Graphics()
      .lineStyle(2, 0x4dfaff, 1, 0, false)
      .drawRoundedRect(0, 0, width, height, radius);
    status.zIndex = 1;
    this.addChild(status);
  }
}
