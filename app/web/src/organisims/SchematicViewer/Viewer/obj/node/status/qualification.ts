import * as PIXI from "pixi.js";

export class QualificationStatus extends PIXI.Container {
  constructor(status?: boolean) {
    super();

    let iconSvg;
    if (status === true) {
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
      d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"\
      clip-rule="evenodd" />\
  </svg>`;
    }

    const texture = PIXI.Texture.from(iconSvg, {
      // scaleMode: PIXI.SCALE_MODES.NEAREST,
    });

    const sprite = new PIXI.Sprite(texture);
    sprite.scale.x = 0.235;
    sprite.scale.y = 0.235;

    this.addChild(sprite);
  }
}
