import { Action } from "./action";
import { ISiStorable } from "./siStorable";
import { Workflow, Step } from "si-registry";
import { Entity } from "./entity";
import { Resource } from "si-entity";
import { Workspace } from "./workspace";

export interface WorkflowContext {
  dryRun: boolean;
  entity?: Entity;
  system?: Entity;
  selection: {
    entity: Entity;
    resource: Resource;
    predecessors: {
      entity: Entity;
      resource: Resource;
    }[];
  };
  strategy?: string;
  failIfMissing?: boolean;
  inputs?: Record<string, any>;
  args?: Record<string, any>;
  output?: Record<string, any>;
  store?: Record<string, any>;
  workspace: Workspace;
}

export enum WorkflowRunState {
  Running = "running",
  Success = "success",
  Failure = "failure",
  Unknown = "unknown",
}

export interface WorkflowRun {
  id: string;
  startUnixTimestamp: number;
  startTimestamp: string;
  endUnixTimestamp?: number;
  endTimestamp?: string;
  state: WorkflowRunState;
  workflowId: string;
  workflowName: string;
  data: Workflow;
  ctx: WorkflowContext;
  siStorable: ISiStorable;
}

export enum WorkflowRunStepState {
  Running = "running",
  Success = "success",
  Failure = "failure",
  Unknown = "unknown",
}

export interface WorkflowRunStep {
  id: string;
  workflowRunId: string;
  startUnixTimestamp: number;
  startTimestamp: string;
  endUnixTimestamp?: number;
  endTimestamp?: string;
  state: WorkflowRunStepState;
  step: Step;
  siStorable: ISiStorable;
}

export enum WorkflowRunStepEntityState {
  Starting = "starting",
  Running = "running",
  Success = "success",
  Failure = "failure",
  Unknown = "unknown",
}

export interface WorkflowRunStepEntity {
  id: string;
  workflowRunId: string;
  workflowRunStepId: string;
  entityId: string;
  startUnixTimestamp: number;
  startTimestamp: string;
  endUnixTimestamp?: number;
  endTimestamp?: string;
  state: WorkflowRunStepEntityState;
  output?: string;
  error?: string;
  siStorable: ISiStorable;
}
