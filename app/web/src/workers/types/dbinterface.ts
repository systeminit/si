import { SqlValue } from "@sqlite.org/sqlite-wasm";
import { WorkspaceMetadata } from "../../api/sdf/dal/workspace";
import {
  ChangeSetId,
} from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";

export interface QueryMeta {
  kind: string,
  workspaceId: string,
  changeSetId: ChangeSetId,
};

export interface Query extends QueryMeta {
  args: Args,
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
  args: Args,
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
type BustCacheFn = (queryKey: string, latestChecksum: string) => void;

export interface DBInterface {
  initDB: () => Promise<void>,
  migrate: () => void,
  initSocket (url: string, bearerToken: string): void,
  initBifrost(url: string, bearerToken: string): void,
  bifrostClose(): void,
  bifrostReconnect(): void,
  get(kind: string, args: Args, checksum: Checksum): Promise<unknown>,
  mjolnir(kind: string, args: Args): void,
  partialKeyFromKindAndArgs (kind: string, args: Args): Promise<QueryKey>, 
  addListenerBustCache(fn: BustCacheFn): void,
  bootstrapChecksums(changeSetId: ChangeSetId): Promise<Record<QueryKey, Checksum>>,
  fullDiagnosticTest(changeSetId: ChangeSetId): void,
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
};

export type RawArgs = Record<string, string>;
// CONSTRAINT: right now there are either zero args (e.g. just workspace & changeset) or 1 (i.e. "the thing", ComponentId, ViewId, et. al)
export class Args {
  #key: string;
  #value: string;
  constructor(args: RawArgs) {
    const entries = Object.entries(args);
    const entry = entries.pop();
    if (entry) {
      this.#key = entry[0];
      this.#value = entry[1];
    } else {
      this.#key = "";
      this.#value = "";
    }
  }

  length() {
    return this.#key === "" ? 0 : 1
  }

  toString() {
    return `${this.#key}|${this.#value}`
  }
};

export type QueryKey = string;  // `kind|argsToString`
export type Checksum = string;  // QueryKey + Checksum is a HIT in sqlite
export type ROWID = number;
export const NOROW = Symbol("NOROW");

interface AbstractAtom {
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  fromSnapshotChecksum: Checksum,
  toSnapshotChecksum: Checksum,
  kind: string,
  origChecksum: Checksum,
  newChecksum: Checksum,
  data: string, // this is a string of JSON we're not parsing
}
export interface RawAtom extends AbstractAtom {
  args: RawArgs,
};

export interface Atom extends AbstractAtom {
  args: Args,
};