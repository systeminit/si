import { Interpreter } from "xstate";

export enum ZoomEventKind {
  ACTIVATE_ZOOMING = "ACTIVATE_ZOOMING",
  INITIATE_ZOOMING = "INITIATE_ZOOMING",
  ZOOMING = "ZOOMING",
  DEACTIVATE_ZOOMING = "DEACTIVATE_ZOOMING",
}

export type ZoomEvent =
  | { type: ZoomEventKind.ACTIVATE_ZOOMING }
  | { type: ZoomEventKind.INITIATE_ZOOMING }
  | { type: ZoomEventKind.ZOOMING }
  | { type: ZoomEventKind.DEACTIVATE_ZOOMING };

export enum ZoomState {
  ZOOMING_ACTIVATED = "zoomingActivated",
  ZOOMING_INITIATED = "zoomingInitiated",
  ZOOMING = "zooming",
}

export function activateZooming(i: Interpreter<unknown>): void {
  i.send(ZoomEventKind.ACTIVATE_ZOOMING);
}

export function initiateZooming(i: Interpreter<unknown>): void {
  i.send(ZoomEventKind.INITIATE_ZOOMING);
}

export function zooming(i: Interpreter<unknown>): void {
  i.send(ZoomEventKind.ZOOMING);
}

export function deactivateZooming(i: Interpreter<unknown>): void {
  i.send(ZoomEventKind.DEACTIVATE_ZOOMING);
}

export function isZoomingActivated(i: Interpreter<unknown>): boolean {
  return i.state.matches(ZoomState.ZOOMING_ACTIVATED);
}

export function isZoomingInitiated(i: Interpreter<unknown>): boolean {
  return i.state.matches(ZoomState.ZOOMING_INITIATED);
}

export function isZooming(i: Interpreter<unknown>): boolean {
  return i.state.matches(ZoomState.ZOOMING);
}
