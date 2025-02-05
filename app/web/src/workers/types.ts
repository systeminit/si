import { WorkspaceMetadata } from "../api/sdf/dal/workspace";
import {
  ChangeSetId,
} from "@/api/sdf/dal/change_set";

export interface QueryMeta {
  kind: string,
  workspaceId: string,
  changeSetId: ChangeSetId,
};

export interface Query extends QueryMeta {
  args: Record<string, string>,
};

export type ENUM_TYPESCRIPT_BINDING = WorkspaceMetadata | null;

export interface QueryResult extends QueryMeta {
  status: "result",
  data: ENUM_TYPESCRIPT_BINDING,
};


export interface QueryMiss extends QueryMeta {
  status: "does_not_exist",
};

export interface PayloadMeta {
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  args: Record<string, string>,
}

export interface UpsertPayload extends PayloadMeta {
  method: "upsert",
  data: ENUM_TYPESCRIPT_BINDING,
}

export interface JSONPatch {
  op: string,
  path: string,
  value: unknown,
}

export interface PatchPayload extends PayloadMeta {
  method: "upsert",
  patches: JSONPatch[],
}

export interface PayloadDelete extends PayloadMeta {
  method: "delete",
}