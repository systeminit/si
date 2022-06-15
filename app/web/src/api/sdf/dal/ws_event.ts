import { HistoryActor } from "@/api/sdf/dal/history_actor";
import { ResourceSyncId } from "@/observable/resource";
import { CodeGenerationId } from "@/observable/code";
import { CheckedQualificationId } from "@/observable/qualification";
import { DependentValuesUpdated } from "@/observable/attribute_value";

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

export interface WsChangeSetCreated extends WsPayload {
  kind: "ChangeSetCreated";
  data: number;
}

export interface WsChangeSetApplied extends WsPayload {
  kind: "ChangeSetApplied";
  data: number;
}

export interface WsChangeSetCanceled extends WsPayload {
  kind: "ChangeSetCanceled";
  data: number;
}

export interface WsEditSessionSaved extends WsPayload {
  kind: "EditSessionSaved";
  data: number;
}

export interface WsResourceSynced extends WsPayload {
  kind: "ResourceSynced";
  data: ResourceSyncId;
}

export interface WsCheckedQualifications extends WsPayload {
  kind: "CheckedQualifications";
  data: CheckedQualificationId;
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

export type WsPayloadKinds =
  | WsEditSessionSaved
  | WsChangeSetCreated
  | WsChangeSetApplied
  | WsChangeSetCanceled
  | WsResourceSynced
  | WsCodeGenerated
  | WsCheckedQualifications
  | WsSecretCreated
  | WsDependentValuesUpdated;
