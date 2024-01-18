// This is a map of valid websocket events to the shape of their payload
// used in the subscribe fn to limit valid event names and set callback payload type

import { ActorView } from "@/api/sdf/dal/history_actor";
import { FuncId } from "@/store/func/funcs.store";
import { ChangeSetId } from "@/store/change_sets.store";
import { ComponentId } from "../components.store";
import { WorkspacePk } from "../workspaces.store";
import { FixStatus } from "../fixes.store";
import {
  AttributeValueId,
  AttributeValueKind,
  AttributeValueStatus,
  StatusUpdatePk,
} from "../status.store";
import { CursorContainerKind } from "../presence.store";
import { UserId } from "../auth.store";

export type WebsocketRequest = CursorRequest | OnlineRequest;

export interface CursorRequest {
  kind: "Cursor";
  data: {
    userName: string;
    userPk: UserId;
    changeSetPk: string | null;
    container: CursorContainerKind;
    containerKey: string | null;
    x: string | null;
    y: string | null;
  };
}

export interface OnlineRequest {
  kind: "Online";
  data: {
    userPk: UserId;
    name: string;
    pictureUrl: string | null;
    idle: boolean;
    changeSetPk: string | null;
  };
}

// TODO: a few of these use the same id objects (ex: componentId)
// but in a few cases the changeset ID may have been accidentally left out?
// once things are working again, we should do a big review of all the realtime events coming from the backend...

export type WsEventPayloadMap = {
  Cursor: {
    x: string | null;
    y: string | null;
    container: string | null;
    containerKey: string | null;
    userPk: string;
    userName: string;
  };
  ChangeSetCreated: string;
  ChangeSetApplied: string;
  ChangeSetWritten: string;
  ChangeSetCancelled: string;

  ChangeSetBeginApprovalProcess: {
    changeSetPk: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetCancelApprovalProcess: {
    changeSetPk: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetMergeVote: {
    changeSetPk: ChangeSetId;
    userPk: UserId;
    vote: string;
  };

  ChangeSetBeginAbandonProcess: {
    changeSetPk: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetCancelAbandonProcess: {
    changeSetPk: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetAbandonVote: {
    changeSetPk: ChangeSetId;
    userPk: UserId;
    vote: string;
  };

  CheckedQualifications: {
    prototypeId: string;
    componentId: string;
  };

  CodeGenerated: {
    componentId: string;
  };

  LogLine: {
    stream: {
      stream: string;
      level: string;
      message: string;
      timestamp: string;
    };
    funcId: FuncId;
    executionKey: string;
  };

  Online: {
    userPk: string;
    name: string;
    pictureUrl: string | null;
    changeSetPk: string | null;
    idle: boolean;
  };

  // NOT CURRENTLY USED - but leaving here so we remember these events exist
  // SecretCreated: number;
  ResourceRefreshed: {
    componentId: string;
  };
  // UpdatedDependentValue: {
  //   componentId: string;
  // }
  // CommandOutput: { runId: string; output: string }
  // CommandReturn: {
  //   runId: string;
  //   resources: Resource[];
  //   output: string[];
  //   runnerState: WorkflowRunnerState;
  // };

  FixReturn: {
    id: string;
    batchId: string;
    attributeValueId: string;
    action: string;
    output: string[];
    status: FixStatus;
  };
  FixBatchReturn: {
    id: string;
    status: FixStatus;
  };
  ComponentCreated: {
    success: boolean;
  };
  ModuleImported: {
    schemaVariantIds: string[];
  };
  WorkspaceImportBeginApprovalProcess: {
    workspacePk: WorkspacePk;
    userPk: UserId;
    createdAt: IsoDateString;
    createdBy: string;
    name: string;
  };
  WorkspaceImportCancelApprovalProcess: {
    workspacePk: WorkspacePk;
    userPk: UserId;
  };
  ImportWorkspaceVote: {
    workspacePk: WorkspacePk;
    userPk: UserId;
    vote: string;
  };
  WorkspaceImported: {
    workspacePk: WorkspacePk;
    userPk: UserId;
  };
  AsyncFinish: {
    id: string;
  };
  AsyncError: {
    id: string;
    error: string;
  };

  // Old fake status update
  // UpdateStatus: {
  //   global: GlobalUpdateStatus;
  //   components?: ComponentUpdateStatus[];
  // };

  StatusUpdate: {
    pk: StatusUpdatePk;
    status: AttributeValueStatus | "statusStarted" | "statusFinished";
    actor: ActorView;
    values: {
      componentId: ComponentId;
      valueId: AttributeValueId;
      valueKind: {
        kind: AttributeValueKind;
        id?: string;
      };
    }[];
  };
};
