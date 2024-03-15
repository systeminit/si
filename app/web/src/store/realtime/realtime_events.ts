// This is a map of valid websocket events to the shape of their payload
// used in the subscribe fn to limit valid event names and set callback payload type

import { ActorView } from "@/api/sdf/dal/history_actor";
import { FuncId } from "@/store/func/funcs.store";
import { DetachedAttributePrototype } from "@/store/asset.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "../components.store";
import { WorkspacePk } from "../workspaces.store";
import { ActionId, ActionStatus } from "../actions.store";
import {
  AttributeValueId,
  AttributeValueKind,
  AttributeValueStatus,
  StatusUpdatePk,
} from "../status.store";
import { CursorContainerKind } from "../presence.store";
import { UserId } from "../auth.store";
import { SecretId } from "../secrets.store";

export type WebsocketRequest = CursorRequest | OnlineRequest;

export interface CursorRequest {
  kind: "Cursor";
  data: {
    userName: string;
    userPk: UserId;
    changeSetId: string | null;
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
    changeSetId: string | null;
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
    changeSetId: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetCancelApprovalProcess: {
    changeSetId: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetMergeVote: {
    changeSetId: ChangeSetId;
    userPk: UserId;
    vote: string;
  };

  ChangeSetBeginAbandonProcess: {
    changeSetId: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetCancelAbandonProcess: {
    changeSetId: ChangeSetId;
    userPk: UserId;
  };
  ChangeSetAbandonVote: {
    changeSetId: ChangeSetId;
    userPk: UserId;
    vote: string;
  };
  ChangeSetAbandoned: {
    changeSetId: ChangeSetId;
    userPk: UserId;
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
    changeSetId: string | null;
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

  ActionRunnerReturn: {
    id: string;
    batchId: string;
    attributeValueId: string;
    action: string;
    output: string[];
    status: ActionStatus;
  };
  ActionBatchReturn: {
    id: string;
    status: ActionStatus;
  };
  ComponentCreated: {
    success: boolean;
    componentId: string;
    changeSetId: string;
  };
  ComponentUpdated: {
    componentId: string;
    changeSetId: string;
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
  ActionAdded: {
    componentId: ComponentId;
    actionId: ActionId;
    changeSetId: ChangeSetId;
  };
  ActionRemoved: {
    componentId: ComponentId;
    actionId: ActionId;
    changeSetId: ChangeSetId;
  };
  SecretUpdated: {
    secretId: SecretId;
    changeSetId: ChangeSetId;
  };
  SecretCreated: {
    secretId: SecretId;
    changeSetId: ChangeSetId;
  };
  SchemaVariantDefinitionCreated: {
    schemaVariantDefinitionId: string;
    changeSetId: ChangeSetId;
  };
  SchemaVariantDefinitionCloned: {
    schemaVariantDefinitionId: string;
    changeSetId: ChangeSetId;
  };
  SchemaVariantDefinitionFinished: {
    taskId: string;
    schemaVariantId: string;
    detachedAttributePrototypes: DetachedAttributePrototype[];
  };
  SchemaVariantDefinitionSaved: {
    schemaVariantDefinitionId: string;
    changeSetId: ChangeSetId;
  };
  FuncCreated: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
  FuncDeleted: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
  FuncReverted: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
  FuncSaved: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
};
