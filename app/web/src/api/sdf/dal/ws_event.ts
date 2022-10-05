import { HistoryActor } from "@/api/sdf/dal/history_actor";
import { ResourceRefreshId } from "@/observable/resource";
import { CodeGenerationId } from "@/observable/code";
import { DependentValuesUpdated } from "@/observable/attribute_value";
import { Resource } from "@/api/sdf/dal/resource";
import { WorkflowRunnerState } from "@/service/workflow/run";

export interface WsEvent<Payload extends WsPayload> {
  version: number;
  billing_account_ids: Array<number>;
  history_actor: HistoryActor;
  payload: Payload;
}

export interface WsPayload {
  kind: string;
  data: unknown;
}

export interface WsResourceRefreshed extends WsPayload {
  kind: "ResourceRefreshed";
  data: ResourceRefreshId;
}

export interface WsDependentValuesUpdated extends WsPayload {
  kind: "UpdatedDependentValue";
  data: DependentValuesUpdated;
}

export interface WsCodeGenerated extends WsPayload {
  kind: "CodeGenerated";
  data: CodeGenerationId;
}

export interface WsSecretCreated extends WsPayload {
  kind: "SecretCreated";
  data: number;
}

export interface WsCommandOutput extends WsPayload {
  kind: "CommandOutput";
  data: { runId: number; output: string };
}

export interface WsCommandReturn extends WsPayload {
  kind: "CommandReturn";
  data: {
    runId: number;
    createdResources: Resource[];
    updatedResources: Resource[];
    output: string[];
    runnerState: WorkflowRunnerState;
  };
}

export type WsPayloadKinds =
  | WsResourceRefreshed
  | WsCodeGenerated
  | WsSecretCreated
  | WsDependentValuesUpdated
  | WsCommandOutput
  | WsCommandReturn;
