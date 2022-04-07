import * as PIXI from "pixi.js";

import * as feather from "feather-icons";

export class QualificationStatus extends PIXI.Container {
  constructor(status?: boolean) {
    super();
    const icon = feather.icons["check-square"];

    let color = "#bbbbbb"; // unknown status
    if (status === true) {
      color = "#86f0ad";
    } else if (status === false) {
      color = "#f08686";
    }

    const texture = PIXI.Texture.from(
      icon.toSvg({
        color,
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
