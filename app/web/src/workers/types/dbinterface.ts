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
  id: Id,
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
  setBearer: (token: string) => void,
  initSocket (url: string, bearerToken: string): void,
  initBifrost(url: string, bearerToken: string): void,
  bifrostClose(): void,
  bifrostReconnect(): void,
  get(changeSetId: ChangeSetId, kind: string, id: Id): Promise<typeof NOROW | AtomDocument>,
  mjolnir(workspaceId: string, changeSetId: ChangeSetId, kind: string, id: Id, checksum?: Checksum): void,
  partialKeyFromKindAndId (kind: string, id: Id): QueryKey, 
  kindAndIdFromKey(key: QueryKey): { kind: string, id: Id},
  addListenerBustCache(fn: BustCacheFn): void,
  atomChecksumsFor(changeSetId: ChangeSetId): Promise<Record<QueryKey, Checksum>>,
  niflheim(workspaceId: string, changeSetId: ChangeSetId): void,
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

export type QueryKey = string;  // `kind|argsToString`
export type Checksum = string;  // QueryKey + Checksum is a HIT in sqlite
export type Id = string;
export type ROWID = number;
export const NOROW = Symbol("NOROW");

interface AbstractAtom {
  id: Id,
  kind: string,
  kindFromChecksum: Checksum,
  kindToChecksum: Checksum,
}
export interface AtomOperation extends AbstractAtom {
  operations: string, // this is a string of JSON
};

export interface AtomMeta {
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  snapshotFromChecksum: Checksum,
  snapshotToChecksum: Checksum,
};

export enum MessageKind {
  PATCH = "PatchMessage",
  MJOLNIR = "MjolnirAtom",
}

export interface PatchAtomMessage {
  meta: AtomMeta,
  kind: MessageKind.PATCH,
  patches: AtomOperation[],
};

export interface AtomMessage {
  kind: MessageKind.MJOLNIR,
  atom: Omit<Atom, "kindFromChecksum">,
  data: object,
};

export interface Atom extends AbstractAtom, AtomMeta {
  operations?: Operation[],
};

// TODO
export type AtomDocument = any;

interface Common {
  kind: string,
  id: Id,
  checksum: string,
};

export interface IndexObject extends Common {
  data: {
    changeSetId: string,
    mvList: Common[];
  },
}
export interface IndexObjectMeta {
  workspaceSnapshotAddress: string,
  frontEndObject: IndexObject,
}