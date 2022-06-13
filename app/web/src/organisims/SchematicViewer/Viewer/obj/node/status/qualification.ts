import * as PIXI from "pixi.js";

export class QualificationStatus extends PIXI.Container {
  constructor(
    status: boolean | null,
    x: number,
    y: number,
    width: number,
    height: number,
  ) {
    super();

    if (status === null) return;

    let iconSvg;
    if (status) {
      const color = "#86f0ad";

      iconSvg = `\
  <svg xmlns="http://www.w3.org/2000/svg" width="64px" height="64px" viewBox="0 0 20 20" fill="${color}">\
    <path\
      fill-rule="evenodd"\
      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"\
      clip-rule="evenodd" />\
  </svg>`;
    } else {
      const color = "#f08686";

      iconSvg = `\
  <svg xmlns="http://www.w3.org/2000/svg" width="64px" height="64px" viewBox="0 0 20 20" fill="${color}">\
    <path\
      fill-rule="evenodd"\
      d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
      clip-rule="evenodd" />\
  </svg>`;
    }

    const texture = PIXI.Texture.from(iconSvg, {
      // scaleMode: PIXI.SCALE_MODES.NEAREST,
    });

    const sprite = new PIXI.Sprite(texture);
    sprite.scale.x = 0.235;
    sprite.scale.y = 0.235;

    sprite.x = x;
    sprite.y = y;
    this.addChild(sprite);

    if (!status) {
      const color = 0xf08686;
      const box = new PIXI.Graphics()
        .lineStyle(1, color, 1, 0, false)
        .drawRoundedRect(-3, -3, width + 6, height + 6, 6);
      box.zIndex = 1;
      this.addChild(box);
    }
  }
}
