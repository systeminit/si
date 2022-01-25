import * as PIXI from "pixi.js";

import * as feather from "feather-icons";

export class ResourceStatus extends PIXI.Container {
  constructor() {
    super();

    const icon = feather.icons["box"];

    const texture = PIXI.Texture.from(
      icon.toSvg({
        color: "#f08686",
        width: 64,
        height: 64,
        "stroke-width": 1.75,
      }),
      {
        // scaleMode: PIXI.SCALE_MODES.NEAREST,
      },
    );

    const sprite = new PIXI.Sprite(texture);
    sprite.scale.x = 0.235;
    sprite.scale.y = 0.235;

    this.addChild(sprite);
  }
}
