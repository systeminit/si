import { Interpreter } from "xstate";

export enum SelectEventKind {
  ACTIVATE_SELECTING = "ACTIVATE_SELECTING",
  INITIATE_SELECTING = "INITIATE_SELECTING",
  SELECTING = "SELECTING",
  DESELECTING = "DESELECTING",
  DEACTIVATE_SELECTING = "DEACTIVATE_SELECTING",
}

export type SelectEvent =
  | { type: SelectEventKind.ACTIVATE_SELECTING }
  | { type: SelectEventKind.INITIATE_SELECTING }
  | { type: SelectEventKind.SELECTING }
  | { type: SelectEventKind.DESELECTING }
  | { type: SelectEventKind.DEACTIVATE_SELECTING };

export enum SelectState {
  SELECTING_ACTIVATED = "selectingActivated",
  SELECTING_INITIATED = "selectingInitiated",
  SELECTING = "selecting",
  DESELECTING = "deselecting",
}

export function activateSelecting(i: Interpreter<unknown>): void {
  i.send(SelectEventKind.ACTIVATE_SELECTING);
}

export function initiateSelecting(i: Interpreter<unknown>): void {
  i.send(SelectEventKind.INITIATE_SELECTING);
}

export function readySelecting(i: Interpreter<unknown>): void {
  i.send(SelectEventKind.ACTIVATE_SELECTING);
  i.send(SelectEventKind.INITIATE_SELECTING);
}

export function selecting(i: Interpreter<unknown>): void {
  i.send(SelectEventKind.SELECTING);
}

export function deSelecting(i: Interpreter<unknown>): void {
  i.send(SelectEventKind.DESELECTING);
}

export function deactivateSelecting(i: Interpreter<unknown>): void {
  i.send(SelectEventKind.DEACTIVATE_SELECTING);
}

export function isSelecting(i: Interpreter<unknown>): boolean {
  return i.state.matches(SelectState.SELECTING);
}

export function isDeselecting(i: Interpreter<unknown>): boolean {
  return i.state.matches(SelectState.DESELECTING);
}
