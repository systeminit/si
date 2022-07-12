import * as PIXI from "pixi.js";

export class Header extends PIXI.Container {
  constructor(
    textString: string,
    width: number,
    height: number,
    color: number,
  ) {
    super();

    this.interactive = false;

    const background = new HeaderBackground(width, height, color);
    background.zIndex = 0;
    this.addChild(background);

    const text = new HeaderText(textString, width, height);
    text.zIndex = 1;
    this.addChild(text);
  }
}

class HeaderBackground extends PIXI.Graphics {
  constructor(width: number, height: number, color: number) {
    super();

    this.beginFill(color);
    this.drawRoundedRect(0, 0, width, height, 0);
    this.endFill();
  }
}

class HeaderText extends PIXI.Text {
  constructor(textString: string, width: number, height: number) {
    const textStyle = new PIXI.TextStyle({
      fontFamily: "Inter",
      fontSize: 12, // TODO: consider rem.
      fontWeight: "400",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    super(textString, textStyle);

    // WebGl text tends to be blurry
    // this.texture.baseTexture.scaleMode = PIXI.SCALE_MODES.NEAREST;
    // TODO set this correctly at the right place.
    this.resolution = window.devicePixelRatio * 2;

    this.position.x = width * 0.5 - this.width * 0.5;

    const offset = (height - this.height) * 0.5;
    this.position.y = offset;
  }
}
