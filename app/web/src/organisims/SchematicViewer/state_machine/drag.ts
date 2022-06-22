import { Interpreter } from "xstate";

export enum DragEventKind {
  ACTIVATE_DRAGGING = "ACTIVATE_DRAGGING",
  INITIATE_DRAGGING = "INITIATE_DRAGGING",
  DRAGGING = "DRAGGING",
  DEACTIVATE_DRAGGING = "DEACTIVATE_DRAGGING",
}

export type DragEvent =
  | { type: DragEventKind.ACTIVATE_DRAGGING }
  | { type: DragEventKind.INITIATE_DRAGGING }
  | { type: DragEventKind.DRAGGING }
  | { type: DragEventKind.DEACTIVATE_DRAGGING };

export enum DragState {
  DRAGGING_ACTIVATED = "draggingActivated",
  DRAGGING_INITIATED = "draggingInitiated",
  DRAGGING = "dragging",
}

export function activateDragging(i: Interpreter<unknown>): void {
  i.send(DragEventKind.ACTIVATE_DRAGGING);
}

export function initiateDragging(i: Interpreter<unknown>): void {
  i.send(DragEventKind.INITIATE_DRAGGING);
}

export function dragging(i: Interpreter<unknown>): void {
  i.send(DragEventKind.DRAGGING);
}

export function deactivateDragging(i: Interpreter<unknown>): void {
  i.send(DragEventKind.DEACTIVATE_DRAGGING);
}

export function isDraggingActivated(i: Interpreter<unknown>): boolean {
  return i.state.matches(DragState.DRAGGING_ACTIVATED);
}

export function isDraggingInitiated(i: Interpreter<unknown>): boolean {
  return i.state.matches(DragState.DRAGGING_INITIATED);
}

export function isDragging(i: Interpreter<unknown>): boolean {
  return i.state.matches(DragState.DRAGGING);
}
