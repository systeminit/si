export type ActionPrototypeId = string;
export type ActionId = string;

export enum ActionState {
  Dispatched = "Dispatched",
  Failed = "Failed",
  OnHold = "OnHold",
  Queued = "Queued",
  Running = "Running",
}

export enum ActionKind {
  Create = "Create",
  Destroy = "Destroy",
  Refresh = "Refresh",
  Manual = "Manual",
  Update = "Update",
}

export interface ActionPrototype {
  id: ActionPrototypeId;
  name: string;
  displayName: string;
}

export enum ActionResultState {
  Success = "Success",
  Failure = "Failure",
  Unknown = "Unknown",
}
