import * as PIXI from "pixi.js";

export class NodeTitle extends PIXI.Container {
  constructor(textString: string, nodeWidth: number) {
    super();

    // this.setZIndex();
    this.disableInteraction();

    const background = new TitleBackground(nodeWidth);
    background.zIndex = 0;
    this.addChild(background);

    const text = new TitleText(textString, nodeWidth);
    text.zIndex = 1;
    this.addChild(text);
  }
  disableInteraction(): void {
    this.interactive = false;
  }
}

class TitleText extends PIXI.Text {
  constructor(textString: string, nodeWidth: number) {
    const textStyle = new PIXI.TextStyle({
      fontFamily: "Source Code Pro",
      fontSize: 12,
      fontWeight: "400",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    super(textString, textStyle);

    this.position.x = nodeWidth * 0.5 - this.width * 0.5;
    this.position.y = 5;
  }
}

class TitleBackground extends PIXI.Graphics {
  constructor(nodeWidth: number) {
    super();

    const backgroundHeight = 22;

    this.beginFill(0x05b1bc);
    this.drawRoundedRect(0, 0, nodeWidth, backgroundHeight, 0);
    this.endFill();
  }
}
