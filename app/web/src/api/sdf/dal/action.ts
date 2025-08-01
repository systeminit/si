import { FuncRunId } from "@/newhotness/api_composables/func_run";
import { ChangeSetId } from "./change_set";
import { ComponentId } from "./component";

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

export type ActionResultState = "Success" | "Failure" | "Unknown";

export interface ActionView {
  id: ActionId;
  actor?: string; // TODO i dont see this on the backend
  prototypeId: ActionPrototypeId;
  componentId: ComponentId | null;
  name: string;
  description?: string;
  kind: ActionKind;
  originatingChangeSetId: ChangeSetId;
  funcRunId?: FuncRunId;
}

export interface ActionProposedView extends ActionView {
  state: ActionState;
  myDependencies: ActionId[];
  dependentOn: ActionId[];
  holdStatusInfluencedBy: ActionId[];
  componentSchemaName?: string;
  componentName?: string;
}
