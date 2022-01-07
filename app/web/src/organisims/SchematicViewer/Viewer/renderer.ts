import * as PIXI from "pixi.js";

export interface RendererSettings {
  view: HTMLCanvasElement;
  resolution: number;
  width: number;
  height: number;
  autoDensity: boolean;
  antialias: boolean;
  backgroundColor: number;
}

export class Renderer extends PIXI.Renderer {
  stage: PIXI.Container;

  constructor(settings: RendererSettings) {
    super({
      view: settings.view,
      resolution: settings.resolution,
      width: settings.width,
      height: settings.height,
      autoDensity: settings.autoDensity,
      antialias: settings.antialias,
      backgroundColor: settings.backgroundColor,
    });

    this.stage = new PIXI.Container();
    this.stage.sortableChildren = true;
    this.stage.interactive = true;
  }

  renderStage(): void {
    this.render(this.stage);
  }

  renderGroup(group: PIXI.DisplayObject): void {
    this.render(group);
  }
}
