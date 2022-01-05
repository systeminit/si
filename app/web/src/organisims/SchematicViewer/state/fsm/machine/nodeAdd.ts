import { Interpreter } from "xstate";

export enum NodeAddEventKind {
  ACTIVATE_NODEADD = "ACTIVATE_NODEADD",
  INITIATE_NODEADD = "INITIATE_NODEADD",
  ADDING_NODE = "ADDING_NODE",
  DEACTIVATE_NODEADD = "DEACTIVATE_NODEADD",
}

export type NodeAddEvent =
  | { type: NodeAddEventKind.ACTIVATE_NODEADD }
  | { type: NodeAddEventKind.INITIATE_NODEADD }
  | { type: NodeAddEventKind.ADDING_NODE }
  | { type: NodeAddEventKind.DEACTIVATE_NODEADD };

export enum NodeAddState {
  NODEADD_ACTIVATED = "nodeAddActivated",
  NODEADD_INITIATED = "nodeAddInitiated",
  ADDING_NODE = "addingNode",
}

export function activateNodeAdd(i: Interpreter<any>): void {
  i.send(NodeAddEventKind.ACTIVATE_NODEADD);
}

export function initiateNodeAdd(i: Interpreter<any>): void {
  i.send(NodeAddEventKind.INITIATE_NODEADD);
}

export function addingNode(i: Interpreter<any>): void {
  i.send(NodeAddEventKind.ADDING_NODE);
}

export function deactivateNodeAdd(i: Interpreter<any>): void {
  i.send(NodeAddEventKind.DEACTIVATE_NODEADD);
}

export function isNodeAddActivated(i: Interpreter<any>): boolean {
  return i.state.matches(NodeAddState.NODEADD_ACTIVATED);
}

export function isNodeAddInitiated(i: Interpreter<any>): boolean {
  return i.state.matches(NodeAddState.NODEADD_INITIATED);
}

export function isAddingNode(i: Interpreter<any>): boolean {
  return i.state.matches(NodeAddState.ADDING_NODE);
}
