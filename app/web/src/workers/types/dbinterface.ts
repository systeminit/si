import { SqlValue } from "@sqlite.org/sqlite-wasm";
import { Operation } from "fast-json-patch";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";
import { WorkspaceMetadata } from "../../api/sdf/dal/workspace";

export interface QueryMeta {
  kind: string;
  workspaceId: string;
  changeSetId: ChangeSetId;
}

export interface Query extends QueryMeta {
  id: Id;
}

export type ENUM_TYPESCRIPT_BINDING = WorkspaceMetadata | null;

export interface QueryResult extends QueryMeta {
  status: "result";
  data: ENUM_TYPESCRIPT_BINDING;
}

export interface QueryMiss extends QueryMeta {
  status: "does_not_exist";
}

export type Column = string;
export type Columns = Column[];
export type BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: string,
  id: string,
) => void;

export interface DBInterface {
  initDB: () => Promise<void>;
  migrate: () => void;
  setBearer: (token: string) => void;
  initSocket(): void;
  initBifrost(): void;
  bifrostClose(): void;
  bifrostReconnect(): void;
  get(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: string,
    id: Id,
  ): Promise<typeof NOROW | AtomDocument>;
  mjolnir(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: string,
    id: Id,
    checksum?: Checksum,
  ): void;
  partialKeyFromKindAndId(kind: string, id: Id): QueryKey;
  kindAndIdFromKey(key: QueryKey): { kind: string; id: Id };
  addListenerBustCache(fn: BustCacheFn): void;
  atomChecksumsFor(
    changeSetId: ChangeSetId,
  ): Promise<Record<QueryKey, Checksum>>;
  changeSetExists(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<boolean>;
  niflheim(workspaceId: string, changeSetId: ChangeSetId): void;
  pruneAtomsForClosedChangeSet(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
  ): void;
  // fullDiagnosticTest(): void;
  odin(): object;
}

export class Ragnarok extends Error {
  workspaceId: string;
  changeSetId: string;

  constructor(message: string, workspaceId: string, changeSetId: string) {
    super(message);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
  }
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
    });
    if ("id" in row) results.push(row as RowWithColumnsAndId);
    else results.push(row as RowWithColumns);
  });
  return results;
};

export type QueryKey = string; // `kind|argsToString`
export type Checksum = string; // QueryKey + Checksum is a HIT in sqlite
export type Id = string;
export type ROWID = number;
export const NOROW = Symbol("NOROW");

interface AbstractAtom {
  id: Id;
  kind: string;
  fromChecksum?: Checksum;
  toChecksum: Checksum;
}
export interface AtomOperation extends AbstractAtom {
  patch: Operation[];
}

export interface AtomMeta {
  workspaceId: WorkspacePk;
  changeSetId: ChangeSetId;
  snapshotFromAddress?: Checksum;
  snapshotToAddress: Checksum;
}

export enum MessageKind {
  PATCH = "PatchMessage",
  MJOLNIR = "MjolnirAtom",
}

export interface PatchBatch {
  meta: AtomMeta;
  kind: MessageKind.PATCH;
  patches: AtomOperation[];
}

export interface AtomMessage {
  kind: MessageKind.MJOLNIR;
  atom: Atom;
  data: object;
}

export interface Atom extends AbstractAtom, AtomMeta {
  operations?: Operation[];
}
interface Common {
  kind: string;
  id: Id;
  checksum: string;
}

export interface IndexObject extends Common {
  data: {
    changeSetId: string;
    mvList: Common[];
  };
}
export interface IndexObjectMeta {
  workspaceSnapshotAddress: string;
  frontEndObject: IndexObject;
}

// TODO
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type AtomDocument = any;

// FAKING IT
export interface BifrostView {
  id: string;
  name: string;
  isDefault: boolean;
  created_at: string;
  updated_at: string;
}

export interface BifrostViewList {
  id: string;
  views: BifrostView[];
}

interface Reference {
  id: string;
  checksum: string;
  kind: string;
}

export interface RawViewList {
  id: string;
  views: Reference[];
}
