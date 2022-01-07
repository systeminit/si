import * as PIXI from "pixi.js";

export class ConnectorLabel extends PIXI.Text {
  constructor(textString: string) {
    const textStyle = new PIXI.TextStyle({
      fontFamily: "Source Code Pro",
      fontSize: 9,
      fontWeight: "300",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    super(textString, textStyle);
    this.setPosition();
    this.setZIndex();
    this.disableInteraction();
  }

  setPosition(): void {
    this.position.x = 10;
    this.position.y = -5;
  }

  setZIndex(): void {
    this.zIndex = 2;
  }

  disableInteraction(): void {
    this.interactive = false;
  }
}
