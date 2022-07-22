import * as PIXI from "pixi.js";
import * as feather from "feather-icons";
import { ResourceHealth } from "@/api/sdf/dal/resource";

export class ResourceStatus extends PIXI.Container {
  constructor(health: ResourceHealth | null) {
    super();

    const icon = feather.icons["box"];

    let color = "#bbbbbb"; // unknown status
    if (health === ResourceHealth.Ok) {
      color = "#86f0ad";
    } else if (health === ResourceHealth.Warning) {
      color = "#f0d286";
    } else if (health === ResourceHealth.Error) {
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
