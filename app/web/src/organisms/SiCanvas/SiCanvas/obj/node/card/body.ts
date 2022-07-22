import * as PIXI from "pixi.js";

export class Body extends PIXI.Container {
  constructor(textString: string, width: number, height: number) {
    super();

    this.interactive = false;

    const background = new ContentBackground(width, height);
    background.zIndex = 0;
    this.addChild(background);

    const text = new ContentText(textString, width);
    text.zIndex = 1;
    this.addChild(text);
  }
}

class ContentBackground extends PIXI.Graphics {
  constructor(width: number, height: number) {
    super();

    this.beginFill(0x282e30);
    this.drawRoundedRect(0, 0, width, height, 0);
    this.endFill();
  }
}

class ContentText extends PIXI.Text {
  constructor(textString: string, width: number) {
    const textStyle = new PIXI.TextStyle({
      fontFamily: "Inter",
      fontSize: 10,
      fontWeight: "400",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    super(textString, textStyle);

    this.position.x = width * 0.5 - this.width * 0.5;
    this.resolution = window.devicePixelRatio * 2;

    const offset = 10;
    this.position.y = offset;
  }
}
