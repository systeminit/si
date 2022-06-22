import * as PIXI from "pixi.js";
import * as Rx from "rxjs";

import { Renderer } from "../renderer";

export class ZoomingManager {
  scale: number;
  sensitivity: number;
  factor: number;
  min: number;
  max: number;
  container: PIXI.Container;
  renderer: Renderer;
  zoomMagnitude$: Rx.ReplaySubject<number | null>;
  zoomFactor$: Rx.ReplaySubject<number | null>;

  constructor(
    container: PIXI.Container,
    renderer: Renderer,
    zoomMagnitude$: Rx.ReplaySubject<number | null>,
    zoomFactor$: Rx.ReplaySubject<number | null>,
  ) {
    this.renderer = renderer;

    this.scale = 1;
    this.sensitivity = 0.001;
    this.factor = 1;
    this.min = 0.5;
    this.max = 1;

    this.container = container;

    this.zoomMagnitude$ = zoomMagnitude$;
    this.zoomFactor$ = zoomFactor$;
  }

  zoom(e: WheelEvent): void {
    const mouseScrollAmount = e.deltaY * this.sensitivity;

    let zoomFactor = this.factor + mouseScrollAmount;
    zoomFactor = Math.min(this.max, Math.max(this.min, zoomFactor));

    const zoomDeltaPercentage = 1 - zoomFactor / this.factor;
    const magnitude = zoomDeltaPercentage;

    const mouseCanvasPosition = {
      x: this.renderer.plugins.interaction.mouse.global.x,
      y: this.renderer.plugins.interaction.mouse.global.y,
    };
    const mousePosition = {
      x: mouseCanvasPosition.x - this.container.position.x,
      y: mouseCanvasPosition.y - this.container.position.y,
    };

    const translation = {
      x: this.container.position.x + mousePosition.x * magnitude,
      y: this.container.position.y + mousePosition.y * magnitude,
    };

    this.factor = zoomFactor;

    this.container.scale.x = zoomFactor;
    this.container.scale.y = zoomFactor;

    this.container.x = translation.x;
    this.container.y = translation.y;
    this.container.updateTransform();

    this.zoomMagnitude$.next(magnitude);
    this.zoomFactor$.next(zoomFactor);
  }
}
