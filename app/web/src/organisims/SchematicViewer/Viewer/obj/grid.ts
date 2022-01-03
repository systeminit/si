import * as PIXI from "pixi.js";

import * as SHADER from "../shader";

export const BACKGROUND_GRID_NAME = "backgroundGrid";

export class Grid extends PIXI.Container {
  shader?: string;
  zoomFactor?: number;
  quad: PIXI.Mesh<PIXI.Shader>;

  constructor(size: number) {
    super();

    this.name = BACKGROUND_GRID_NAME;
    this.zoomFactor = 1;

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
      uZoomFactor: this.zoomFactor,
    };

    const shader = PIXI.Shader.from(
      SHADER.gridVertexShader,
      SHADER.gridFragmentShader,
      uniforms,
    );

    this.quad = new PIXI.Mesh(geo, shader);
    this.quad.name = "quad";
    this.quad.blendMode = PIXI.BLEND_MODES.NORMAL_NPM;
    this.quad.position.set(x, y);
    this.quad.scale.set(1);
    this.addChild(this.quad);
  }

  updateZoomFactor(zoomFactor: number) {
    const c = this.getChildByName("quad") as PIXI.Mesh<PIXI.Shader>;
    c.shader.uniforms.uZoomFactor = zoomFactor;
  }
}
