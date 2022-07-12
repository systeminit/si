import * as PIXI from "pixi.js";

import * as SHADER from "../shader";

export const BACKGROUND_GRID_NAME = "backgroundGrid";

export class Grid extends PIXI.Container {
  shader?: string;
  zoomFactor?: number;
  quad: PIXI.Mesh<PIXI.Shader>;

  constructor(
    rendererWidth: number,
    rendererHeight: number,
    lightMode: boolean,
  ) {
    super();

    this.name = BACKGROUND_GRID_NAME;
    this.zoomFactor = 1;

    const side = Math.max(rendererWidth, rendererHeight) * 2.0;

    const v1 = { x: -side, y: -side };
    const v2 = { x: side, y: -side };
    const v3 = { x: side, y: side };
    const v4 = { x: -side, y: side };
    const geo = new PIXI.Geometry()
      .addAttribute(
        "aVertexPosition",
        [v1.x, v1.y, v2.x, v2.y, v3.x, v3.y, v4.x, v4.y],
        2,
      )
      .addAttribute("aUvs", [0, 0, 1, 0, 1, 1, 0, 1], 2)
      .addIndex([0, 1, 2, 0, 3, 2]); // two triangles...

    const uniforms = {
      // The color of the grid lines.
      uColor: [0.198, 0.198, 0.198],
      uBorderThickness: 0.02,
      uGridSubdivisions: side / 10.0,
      uZoomFactor: this.zoomFactor,
    };

    // If in light mode, ensure the grid line color is white.
    // If we eventually implement a style sheet, we can also convert
    // hex colors directly: "PIXI.utils.hex2rgb(0xffffff)"
    if (lightMode) {
      uniforms.uColor = [1.0, 1.0, 1.0];
    }

    const shader = PIXI.Shader.from(
      SHADER.gridVertexShader,
      SHADER.gridFragmentShader,
      uniforms,
    );

    this.quad = new PIXI.Mesh(geo, shader);
    this.quad.name = "quad";
    this.quad.blendMode = PIXI.BLEND_MODES.NORMAL_NPM;
    this.quad.scale.set(1);
    this.addChild(this.quad);
  }

  updateZoomFactor(zoomFactor: number) {
    const c = this.getChildByName("quad") as PIXI.Mesh<PIXI.Shader>;
    // clamp(zoomFactor, 0.1, 1.);
    c.shader.uniforms.uZoomFactor = Math.max(Math.min(zoomFactor, 1), 0.1);
  }
}
