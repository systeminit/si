import { SqlValue } from "@sqlite.org/sqlite-wasm";
import { WorkspaceMetadata } from "../../api/sdf/dal/workspace";
import {
  ChangeSetId,
} from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";
import { Operation } from "fast-json-patch";

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

export type Column = string;
export type Columns = Column[];
type BustCacheFn = (queryKey: string, latestChecksum: string) => void;

export interface DBInterface {
  initDB: () => Promise<void>,
  migrate: () => void,
  initSocket (url: string, bearerToken: string): void,
  initBifrost(url: string, bearerToken: string): void,
  bifrostClose(): void,
  bifrostReconnect(): void,
  get(changeSetId: ChangeSetId, kind: string, args: Args): Promise<typeof NOROW | AtomDocument>,
  mjolnir(changeSetId: ChangeSetId, kind: string, args: Args): void,
  partialKeyFromKindAndArgs (kind: string, args: Args): QueryKey, 
  kindAndArgsFromKey(key: QueryKey): { kind: string, args: Args},
  addListenerBustCache(fn: BustCacheFn): void,
  bootstrapChecksums(changeSetId: ChangeSetId): Promise<Record<QueryKey, Checksum>>,
  fullDiagnosticTest(): void,
}

export type RealSqlValue = NonNullable<SqlValue>;
export type RowWithColumns = Record<Column, SqlValue>;
export type RowID = Record<"id", number>;
export type RowWithColumnsAndId = RowID & RowWithColumns;
export type Records = (RowWithColumns | RowWithColumnsAndId)[];

export const interpolate = (columns: Columns, rows: SqlValue[][]): Records => {
  const results: Records = [];
  rows.forEach((values) => {
    const row: RowWithColumns = {};
    columns.forEach((column, idx) => {
      const val = values[idx];
      if (val) row[column] = val;
    })
    if ("id" in row)
      results.push(row as RowWithColumnsAndId);
    else
      results.push(row as RowWithColumns);
  })
  return results;
};

export type RawArgs = Record<string, string>;
// CONSTRAINT: right now there are either zero args (e.g. just workspace & changeset) or 1 (i.e. "the thing", ComponentId, ViewId, et. al)
export class Args {
  #key: string;
  #value: string;
  #raw: RawArgs;
  constructor(args: RawArgs) {
    this.#raw = args;
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

  raw() {
    return this.#raw;
  }

  length() {
    return this.#key === "" ? 0 : 1
  }

  toString() {
    return `${this.#key}|${this.#value}`
  }

  static fromString(key: QueryKey) {
    const parts = key.split('|');
    if (parts.length !== 2) throw new Error("Bad parts")
    const _key = parts[0];
    const value = parts[1];
    if (!_key || !value) throw new Error("Missing parts")
    const raw: RawArgs = {_key: value};
    return new Args(raw);
  }
};

export type QueryKey = string;  // `kind|argsToString`
export type Checksum = string;  // QueryKey + Checksum is a HIT in sqlite
export type ROWID = number;
export const NOROW = Symbol("NOROW");

interface AbstractAtom {
  kind: string,
  kindFromChecksum: Checksum,
  kindToChecksum: Checksum,
}
export interface AtomOperation extends AbstractAtom {
  args: RawArgs,
  operations: string, // this is a string of JSON
};

export interface AtomMeta {
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  snapshotFromChecksum: Checksum,
  snapshotToChecksum: Checksum,
};

export interface AtomMessage {
  meta: AtomMeta,
  atoms: AtomOperation[],
};

export interface Atom extends AbstractAtom, AtomMeta {
  args: Args,
  operations: Operation[],
};

// TODO
export type AtomDocument = any;