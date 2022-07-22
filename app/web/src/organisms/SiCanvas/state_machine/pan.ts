import { Interpreter } from "xstate";

export enum PanEventKind {
  ACTIVATE_PANNING = "ACTIVATE_PANNING",
  INITIATE_PANNING = "INITIATE_PANNING",
  PANNING = "PANNING",
  DEACTIVATE_PANNING = "DEACTIVATE_PANNING",
}

export type PanEvent =
  | { type: PanEventKind.ACTIVATE_PANNING }
  | { type: PanEventKind.INITIATE_PANNING }
  | { type: PanEventKind.PANNING }
  | { type: PanEventKind.DEACTIVATE_PANNING };

export enum PanState {
  PANNING_ACTIVATED = "panningActivated",
  PANNING_INITIATED = "panningInitiated",
  PANNING = "panning",
}

export function activatePanning(i: Interpreter<unknown>): void {
  i.send(PanEventKind.ACTIVATE_PANNING);
}

export function initiatePanning(i: Interpreter<unknown>): void {
  i.send(PanEventKind.INITIATE_PANNING);
}

export function readyPanning(i: Interpreter<unknown>): void {
  i.send(PanEventKind.ACTIVATE_PANNING);
  i.send(PanEventKind.INITIATE_PANNING);
}

export function panning(i: Interpreter<unknown>): void {
  i.send(PanEventKind.PANNING);
}

export function deactivatePanning(i: Interpreter<unknown>): void {
  i.send(PanEventKind.DEACTIVATE_PANNING);
}

export function isPanningActivated(i: Interpreter<unknown>): boolean {
  return i.state.matches(PanState.PANNING_ACTIVATED);
}

export function isPanningInitiated(i: Interpreter<unknown>): boolean {
  return i.state.matches(PanState.PANNING_INITIATED);
}

export function isPanning(i: Interpreter<unknown>): boolean {
  return i.state.matches(PanState.PANNING);
}
