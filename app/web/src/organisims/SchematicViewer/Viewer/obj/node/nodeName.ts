import * as PIXI from "pixi.js";

export class NodeName extends PIXI.Text {
  constructor(textString: string, nodeWidth: number) {
    const textStyle = new PIXI.TextStyle({
      fontFamily: "Roboto",
      fontSize: 10,
      fontWeight: "400",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    super(textString, textStyle);
    this.resolution = window.devicePixelRatio * 2;
    this.setPosition(nodeWidth);
    this.setZIndex();
    this.disableInteraction();
  }

  setPosition(nodeWidth: number): void {
    this.position.x = nodeWidth * 0.5 - this.width * 0.5;
    this.position.y = 25;
  }

  setZIndex(): void {
    this.zIndex = 2;
  }

  disableInteraction(): void {
    this.interactive = false;
  }
}
