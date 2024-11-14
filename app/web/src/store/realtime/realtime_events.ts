// This is a map of valid websocket events to the shape of their payload
// used in the subscribe fn to limit valid event names and set callback payload type

import { IRect } from "konva/lib/types";
import { FuncBinding, FuncId, FuncSummary } from "@/api/sdf/dal/func";
import {
  ChangeSetId,
  ChangeSetStatus,
  ChangeSet,
} from "@/api/sdf/dal/change_set";
import { ComponentId, RawComponent, RawEdge } from "@/api/sdf/dal/component";
import {
  ComponentType,
  SchemaVariant,
  SchemaId,
  SchemaVariantId,
} from "@/api/sdf/dal/schema";
import { ActionId } from "@/api/sdf/dal/action";
import { WorkspacePk } from "../workspaces.store";
import { StatusUpdate } from "../status.store";
import { CursorContainerKind } from "../presence.store";
import { UserId } from "../auth.store";
import { SecretId } from "../secrets.store";
import { FuncRunId } from "../actions.store";
import { AwsCliCommand } from "../func/funcs.store";
import { FuncRunLogId } from "../func_runs.store";

export type WebsocketRequest =
  | CursorRequest
  | OnlineRequest
  | ComponentPositionRequest;

export interface CursorRequest {
  kind: "Cursor";
  data: {
    userName: string;
    userPk: UserId;
    changeSetId: string | null;
    viewId: string | null;
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
    viewId: string | null;
  };
}

export interface ComponentPositionRequest {
  kind: "ComponentSetPosition";
  data: {
    clientUlid?: string;
    viewId: string;
    changeSetId: string | null;
    positions: ({ componentId: ComponentId } & IRect)[];
  };
}

// TODO: a few of these use the same id objects (ex: componentId)
// but in a few cases the change set ID may have been accidentally left out?
// once things are working again, we should do a big review of all the realtime events coming from the backend...

export type WsEventPayloadMap = {
  Cursor: {
    x: string | null;
    y: string | null;
    container: string | null;
    containerKey: string | null;
    userPk: string;
    userName: string;
    changeSetId: string | null;
    viewId: string | null;
  };
  ChangeSetCreated: string;
  ChangeSetWritten: string;
  ChangeSetCancelled: string;
  Conflict: string;

  SetComponentPosition: {
    changeSetId: ChangeSetId;
    clientUlid: string;
    viewId: string;
    positions: ({ componentId: ComponentId } & IRect)[];
  };

  ChangeSetApplied: {
    changeSetId: ChangeSetId;
    toRebaseChangeSetId: ChangeSetId;
    userPk: UserId;
  };
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
  ChangeSetStatusChanged: {
    fromStatus: ChangeSetStatus;
    changeSet: ChangeSet;
  };
  CheckedQualifications: {
    prototypeId: string;
    componentId: string;
  };

  CodeGenerated: {
    componentId: string;
  };

  Online: {
    userPk: string;
    name: string;
    pictureUrl: string | null;
    changeSetId: string | null;
    viewId: string | null;
    idle: boolean;
  };

  ResourceRefreshed: {
    component: RawComponent;
    changeSetId: string;
  };

  // NOT CURRENTLY USED - but leaving here so we remember these events exist
  // SecretCreated: number;
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

  ComponentCreated: {
    component: RawComponent;
    changeSetId: string;
  };
  ComponentDeleted: {
    componentId: ComponentId;
    changeSetId: string;
  };
  ComponentUpdated: {
    component: RawComponent;
    changeSetId: string;
  };
  ComponentUpgraded: {
    component: RawComponent;
    originalComponentId: ComponentId;
    changeSetId: string;
  };
  InferredEdgeUpsert: {
    changeSetId: string;
    edges: RawEdge[];
  };
  InferredEdgeRemove: {
    changeSetId: string;
    edges: RawEdge[];
  };
  ConnectionUpserted: RawEdge;
  ConnectionDeleted: {
    fromComponentId: string;
    toComponentId: string;
    fromSocketId: string;
    toSocketId: string;
  };
  ManagementFuncExecuted: {
    managerComponentId: string;
    prototypeId: string;
    funcRunId: FuncRunId;
    changeSetId: string;
  };
  ModuleImported: SchemaVariant[];
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

  StatusUpdate: StatusUpdate;

  ActionsListUpdated: {
    changeSetId: ChangeSetId;
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
  SecretDeleted: {
    secretId: SecretId;
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
  SchemaVariantDeleted: {
    schemaVariantId: SchemaVariantId;
    schemaId: SchemaId;
    changeSetId: ChangeSetId;
  };
  SchemaVariantCreated: SchemaVariant;
  SchemaVariantUpdated: SchemaVariant;
  SchemaVariantReplaced: {
    schemaId: SchemaId;
    oldSchemaVariantId: SchemaVariantId;
    newSchemaVariant: SchemaVariant;
    changeSetId: ChangeSetId;
  };
  SchemaVariantCloned: {
    schemaVariantId: string;
    changeSetId: ChangeSetId;
  };
  SchemaVariantUpdateFinished: {
    changeSetId: string;
    oldSchemaVariantId: string;
    newSchemaVariantId: string;
  };
  SchemaVariantSaved: {
    schemaId: string;
    schemaVariantId: string;
    name: string;
    category: string;
    displayName?: string;
    color: string;
    changeSetId: ChangeSetId;
    componentType: ComponentType;
    link?: string;
    description?: string;
  };
  FuncBindingsUpdated: {
    types: string;
    bindings: FuncBinding[];
    changeSetId: ChangeSetId;
  };
  FuncCreated: {
    types: string;
    funcSummary: FuncSummary;
    changeSetId: ChangeSetId;
  };
  FuncUpdated: {
    types: string;
    funcSummary: FuncSummary;
    changeSetId: ChangeSetId;
    clientUlid?: string;
  };
  FuncDeleted: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
  FuncGenerating: {
    funcId: FuncId;
    command: AwsCliCommand;
  };
  FuncCodeSaved: {
    funcCode: {
      funcId: FuncId;
      // it sends the code but, we are not going to touch it
    };
    generated?: boolean;
  };
  FuncSaved: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
  FuncArgumentsSaved: {
    funcId: FuncId;
    changeSetId: ChangeSetId;
  };
  FuncRunLogUpdated: {
    funcRunId: FuncRunId;
    funcRunLogId: FuncRunLogId;
    actionId?: ActionId;
  };
  AuditLogsPublished: {
    changeSetId: ChangeSetId;
    changeSetStatus: ChangeSetStatus;
  };
};
