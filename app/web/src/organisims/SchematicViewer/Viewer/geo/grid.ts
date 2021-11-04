import * as PIXI from "pixi.js";

import * as SHADER from "../shader";


export const BACKGROUND_GRID_NAME = "backgroundGrid";

export class Grid extends PIXI.Container {
  name: string;
  shader?: string;

  constructor(size: number) {
    super();

    this.name = BACKGROUND_GRID_NAME;

    const x = size;
    const y = size;

    const v1 = { x: -x, y: -y };
    const v2 = { x: x, y: -y };
    const v3 = { x: x, y: y };
    const v4 = { x: -x, y: y };

    const geo = new PIXI.Geometry()
      .addAttribute(
        "aVertexPosition",
        [v1.x, v1.y, v2.x, v2.y, v3.x, v3.y, v4.x, v4.y],
        2,
      )
      .addAttribute("aUvs", [0, 0, 1, 0, 1, 1, 0, 1], 2)
      .addIndex([0, 1, 2, 0, 3, 2]); // two triangles...

    const uniforms = {
      uColor: [0.198, 0.198, 0.198],
      uBorderThickness: 0.02,
      uGridSubdivisions: 80.0,
    };

    const shader = PIXI.Shader.from(
      SHADER.gridVertexShader,
      SHADER.gridFragmentShader,
      uniforms,
    );

    const quad = new PIXI.Mesh(geo, shader);
    quad.blendMode = PIXI.BLEND_MODES.NORMAL_NPM;
    quad.position.set(x, y);
    quad.scale.set(1);

    this.addChild(quad);
  }

  // update(zoomFactor: number) {
  //   // update uGridSubdivisions and shader
  // }
}
