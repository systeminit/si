import { Interpreter } from "xstate";

export enum ConnectEventKind {
  ACTIVATE_CONNECTING = "ACTIVATE_CONNECTING",
  INITIATE_CONNECTING = "INITIATE_CONNECTING",
  CONNECTING = "CONNECTING",
  CONNECTING_TO_SOCKET = "CONNECTING_TO_SOCKET",
  DEACTIVATE_CONNECTING = "DEACTIVATE_CONNECTING",
}

export type ConnectEvent =
  | { type: ConnectEventKind.ACTIVATE_CONNECTING }
  | { type: ConnectEventKind.INITIATE_CONNECTING }
  | { type: ConnectEventKind.CONNECTING }
  | { type: ConnectEventKind.CONNECTING_TO_SOCKET }
  | { type: ConnectEventKind.DEACTIVATE_CONNECTING };

export enum ConnectState {
  CONNECTING_ACTIVATED = "connectingActivated",
  CONNECTING_INITIATED = "connectingInitiated",
  CONNECTING = "connecting",
  CONNECTING_TO_SOCKET = "connectingToSocket",
}

export function activateConnecting(i: Interpreter<unknown>): void {
  i.send(ConnectEventKind.ACTIVATE_CONNECTING);
}

export function initiateConnecting(i: Interpreter<unknown>): void {
  i.send(ConnectEventKind.INITIATE_CONNECTING);
}

export function connecting(i: Interpreter<unknown>): void {
  i.send(ConnectEventKind.CONNECTING);
}

export function connectingToSocket(i: Interpreter<unknown>): void {
  i.send(ConnectEventKind.CONNECTING_TO_SOCKET);
}

export function deactivateConnecting(i: Interpreter<unknown>): void {
  i.send(ConnectEventKind.DEACTIVATE_CONNECTING);
}

export function isConnectingActivated(i: Interpreter<unknown>): boolean {
  return i.state.matches(ConnectState.CONNECTING_ACTIVATED);
}

export function isConnectingInitiated(i: Interpreter<unknown>): boolean {
  return i.state.matches(ConnectState.CONNECTING_INITIATED);
}

export function isConnecting(i: Interpreter<unknown>): boolean {
  return i.state.matches(ConnectState.CONNECTING);
}

export function isConnectingToSocket(i: Interpreter<unknown>): boolean {
  return i.state.matches(ConnectState.CONNECTING_TO_SOCKET);
}
