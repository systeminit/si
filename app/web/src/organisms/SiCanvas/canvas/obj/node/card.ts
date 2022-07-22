import * as PIXI from "pixi.js";

import { Header } from "./card/header";
import { Body } from "./card/body";

export class Card extends PIXI.Container {
  constructor(
    width: number,
    height: number,
    radius: number,
    title: string,
    name: string,
    headerColor: number,
  ) {
    super();

    const background = new PIXI.Graphics()
      .beginFill(0x282e30)
      .drawRoundedRect(0, 0, width, height, radius)
      .endFill();

    this.addChild(background);

    const headerHeight = 22;
    const header = new Header(title, width, headerHeight, headerColor);

    header.mask = background;
    header.zIndex = 1;
    this.addChild(header);

    const contentHeight = height - headerHeight;

    const content = new Body(name, width, contentHeight);

    const offset = height - contentHeight;
    content.position.y = offset;

    content.mask = background;
    content.zIndex = 1;

    this.addChild(content);
  }
}
