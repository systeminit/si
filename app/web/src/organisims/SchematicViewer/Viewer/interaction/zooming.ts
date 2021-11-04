import * as PIXI from "pixi.js";

import { zoomMagnitude$ } from "../../state";

export class ZoomingManager {
  data?: PIXI.InteractionData | undefined;
  scale: number;
  sensitivity: number;
  factor: number;
  min: number;
  max: number;
  container: PIXI.Container;

  constructor(container: PIXI.Container) {
    this.scale = 1;
    this.sensitivity = 0.001;
    this.factor = 1;
    this.min = 0.4;
    this.max = 1;

    this.container = container;
  }

  beforePan(data: PIXI.InteractionData): void {
    this.data = data;
  }

  zoom(e: WheelEvent): void {
    const mouseScrollAmount = e.deltaY * this.sensitivity;

    let zoomFactor = this.factor + mouseScrollAmount;
    zoomFactor = Math.min(this.max, Math.max(this.min, zoomFactor));

    const zoomDeltaPercentage = 1 - zoomFactor / this.factor;
    const magnitude = zoomDeltaPercentage;

    const mousePosition = {
      x: e.clientX - this.container.position.x,
      y: e.clientY - this.container.position.y,
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

    zoomMagnitude$.next(magnitude);

    // Observables to pass state back to the interaction manager... ^^
  }
}