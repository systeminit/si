import { SDFError } from "@/api/sdf";
import { refreshActivitySummary$ } from "@/observables";
import Bottle from "bottlejs";
import {
  WorkflowRun,
  WorkflowRunStep,
  WorkflowRunStepEntity,
} from "../model/workflow";

export interface IRunActionRequest {
  workspaceId: string;
  entityId: string;
  systemId: string;
  actionName: string;
}

export interface IRunActionReplySuccess {
  workflowRun: WorkflowRun;
  error?: never;
}

export interface IRunActionReplyFailure {
  workflowRun?: never;
  error: SDFError;
}

export type IRunActionReply = IRunActionReplySuccess | IRunActionReplyFailure;

export async function runAction(
  request: IRunActionRequest,
): Promise<IRunActionReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IRunActionReply = await sdf.post(
    "workflowDal/runAction",
    request,
  );
  return reply;
}

export interface IListActionRequest {
  workspaceId: string;
  entityId: string;
  systemId: string;
  actionName?: string;
}

export interface IListActionReplySuccess {
  workflowRuns: {
    workflowRun: WorkflowRun;
    steps: {
      step: WorkflowRunStep;
      stepEntities: WorkflowRunStepEntity[];
    }[];
  }[];
  error?: never;
}

export interface IListActionReplyFailure {
  workflowRuns?: never;
  error: SDFError;
}

export type IActionListReply =
  | IListActionReplySuccess
  | IListActionReplyFailure;

export async function listAction(
  request: IListActionRequest,
): Promise<IActionListReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IActionListReply = await sdf.get(
    "workflowDal/listAction",
    request,
  );
  return reply;
}

export const WorkflowDal = {
  runAction,
  listAction,
};
