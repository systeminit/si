import { SqlValue } from "@sqlite.org/sqlite-wasm";
import { WorkspaceMetadata } from "../../api/sdf/dal/workspace";
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

type Column = string;
type Columns = Column[];
type BustCacheFn = (key: string) => void;

export interface DBInterface {
  initDB: () => Promise<void>,
  migrate: () => void,
  testRainbowBridge(): Promise<{columns: Columns, rows: SqlValue[][]}>,
  initSocket (url: string, bearerToken: string): void,
  initBifrost(url: string, bearerToken: string): void,
  bifrostClose(): void,
  bifrostReconnect(): void,
  newChangeSet?(changeSetId: ChangeSetId): void,
  newSnapshot?(changeSetId: ChangeSetId, fromChecksum: string, toChecksum: string): void,
  bifrost: {
    pull (changeSetId: ChangeSetId, kind: string, args: Record<string, string>): SqlValue[][]
  },
  addListenerBustCache(fn: BustCacheFn): void,
}

type RowWithColumns = Record<Column, SqlValue>;
type Records = RowWithColumns[];

export const interpolate = (columns: Columns, rows: SqlValue[][]): Records => {
  const results: Records = [];
  rows.forEach((values) => {
    const row: RowWithColumns = {};
    columns.forEach((column, idx) => {
      const val = values[idx];
      if (val) row[column] = val;
    })
    results.push(row);
  })
  return results;
}