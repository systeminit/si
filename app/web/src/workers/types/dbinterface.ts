import {
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecReturnThisOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import { Operation } from "fast-json-patch";
import { Span } from "@opentelemetry/api";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";
import { DefaultMap } from "@/utils/defaultmap";
import {
  BifrostConnection,
  EntityKind,
  PossibleConnection,
} from "./entity_kind_types";

export type Column = string;
export type Columns = Column[];
export type BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
) => void;

export type OutgoingConnections = DefaultMap<
  string,
  Record<string, BifrostConnection>
>;

export type RainbowFn = (changeSetId: ChangeSetId, label: string) => void;
export type LobbyExitFn = () => void;
export interface DBInterface {
  initDB: (testing: boolean) => Promise<void>;
  migrate: (testing: boolean) => void;
  setBearer: (token: string) => void;
  initSocket(): Promise<void>;
  initBifrost(): void;
  bifrostClose(): void;
  bifrostReconnect(): void;
  linkNewChangeset(
    workspaceId: string,
    headChangeSetId: string,
    changeSetId: string,
    workspaceSnapshotAddress: string,
  ): Promise<void>;
  getConnectionByAnnotation(
    workspaceId: string,
    changeSetId: string,
    annotation: string,
  ): {
    exactMatches: Array<PossibleConnection>;
    typeMatches: Array<PossibleConnection>;
    nonMatches: Array<PossibleConnection>;
  };
  getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): OutgoingConnections | undefined;
  get(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: EntityKind,
    id: Id,
  ): Promise<typeof NOROW | AtomDocument>;
  mjolnir(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: EntityKind,
    id: Id,
    checksum?: Checksum,
  ): void;
  partialKeyFromKindAndId(kind: EntityKind, id: Id): QueryKey;
  kindAndIdFromKey(key: QueryKey): { kind: EntityKind; id: Id };
  addListenerBustCache(fn: BustCacheFn): void;
  addListenerInFlight(fn: RainbowFn): void;
  addListenerReturned(fn: RainbowFn): void;
  addListenerLobbyExit(fn: LobbyExitFn): void;
  atomChecksumsFor(
    changeSetId: ChangeSetId,
  ): Promise<Record<QueryKey, Checksum>>;
  changeSetExists(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<boolean>;
  niflheim(workspaceId: string, changeSetId: ChangeSetId): Promise<boolean>;
  pruneAtomsForClosedChangeSet(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
  ): void;
  /* these are used for testing purposes, and should not be used outside the web worker in production code */
  oneInOne(rows: SqlValue[][]): SqlValue | typeof NOROW;
  encodeDocumentForDB(doc: object): Promise<ArrayBuffer>;
  decodeDocumentFromDB(doc: ArrayBuffer): AtomDocument;
  handlePatchMessage(data: PatchBatch, span?: Span): Promise<void>;
  handleHammer(msg: AtomMessage, span?: Span): Promise<void>;
  exec(
    opts: ExecBaseOptions &
      ExecRowModeArrayOptions &
      (ExecReturnThisOptions | ExecReturnResultRowsOptions) & {
        sql: FlexibleString;
      },
  ): SqlValue[][];
  bobby(): Promise<void>;
  ragnarok(
    workspaceId: string,
    changeSetId: string,
    noColdStart?: boolean,
  ): Promise<void>;
  // show me everything
  odin(changeSetId: ChangeSetId): object;
}

export class Ragnarok extends Error {
  workspaceId: string;
  changeSetId: string;
  fromSnapshotAddress: string | undefined;
  snapshotFromAddress: string | undefined;

  constructor(
    message: string,
    workspaceId: string,
    changeSetId: string,
    fromSnapshotAddress: string | undefined,
    snapshotFromAddress: string | undefined,
  ) {
    super(message);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.fromSnapshotAddress = fromSnapshotAddress;
    this.snapshotFromAddress = snapshotFromAddress;
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
  kind: EntityKind;
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
  INDEXUPDATE = "IndexUpdate",
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

export interface IndexUpdate {
  kind: MessageKind.INDEXUPDATE;
  meta: AtomMeta;
}

export interface Atom extends AbstractAtom, AtomMeta {
  operations?: Operation[];
}
interface Common {
  kind: EntityKind;
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

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type AtomDocument = any;
