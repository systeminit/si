import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { ManagementPrototypeId } from "@/api/sdf/dal/func";
import { Timestamp } from "@/api/sdf/dal/timestamp";
import { FuncRunId } from "./func_run";
import { UserPk, WorkspacePk } from "./si_id";

export type ManagementFuncJobStateId = string;

export interface ManagementFuncJobState {
  id: ManagementFuncJobStateId;
  workspaceId: WorkspacePk;
  changeSetId: ChangeSetId;
  componentId: ComponentId;
  prototypeId: ManagementPrototypeId;
  userId?: UserPk;
  funcRunId?: FuncRunId;
  state: ManagementState;
  timestamp: Timestamp;
  message?: string;
}

export type ManagementState =
  | "executing"
  | "failure"
  | "operating"
  | "pending"
  | "success";
