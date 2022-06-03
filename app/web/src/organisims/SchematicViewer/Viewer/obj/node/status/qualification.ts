import * as PIXI from "pixi.js";

import * as feather from "feather-icons";

export class QualificationStatus extends PIXI.Container {
  constructor(
    status?: boolean,
    x: number,
    y: number,
    width: number,
    height: number,
  ) {
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

    sprite.x = x;
    sprite.y = y;
    this.addChild(sprite);

    if (!status) {
      const color = 0xf08686;
      const box = new PIXI.Graphics()
        .lineStyle(2, color, 1, 0, false)
        .drawRoundedRect(-4, -4, width + 8, height + 8, 6);
      box.zIndex = 1;
      this.addChild(box);
    }
  }
}
