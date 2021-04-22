import { Diff } from "./diff";
import { Resource } from "./resource";
import { ISiStorable } from "./siStorable";

export enum ActionState {
  Running = "running",
  Success = "success",
  Failure = "failure",
  Unknown = "unknown",
}

export interface Action {
  id: string;
  name: string;
  dryRun: boolean;
  state: ActionState;
  resource?: Resource;
  resourceDiff?: Diff;
  startUnixTimestamp: number;
  startTimestamp: string;
  endUnixTimestamp?: number;
  endTimestamp?: string;
  output?: string;
  error?: string;
  entityId: string;
  systemId: string;
  workflowRunId: string;
  siStorable: ISiStorable;
}
