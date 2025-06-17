import * as Comlink from "comlink";
import {
  Database,
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
import { ComponentId } from "@/api/sdf/dal/component";
import {
  Connection,
  EntityKind,
  PossibleConnection,
  Prop,
} from "./entity_kind_types";

export type Column = string;
export type Columns = Column[];
export type BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  noBroadcast?: boolean,
) => void;

export type OutgoingConnections = DefaultMap<
  string,
  Record<string, Connection>
>;

export type UpdateFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  doc: AtomDocument,
  listIds: string[],
  removed: boolean,
  noBroadcast?: boolean,
) => void;
export type RainbowFn = (
  changeSetId: ChangeSetId,
  label: string,
  noBroadcast?: boolean,
) => void;
export type LobbyExitFn = (
  workspacePk: string,
  changeSetId: string,
  noBroadcast?: boolean,
) => void;

export type MjolnirBulk = Array<{
  kind: string;
  id: Id;
  checksum?: Checksum;
}>;

export const SHARED_BROADCAST_CHANNEL_NAME = "SHAREDWORKER_BROADCAST";

export type BroadcastMessage =
  | {
      messageKind: "cacheBust";
      arguments: {
        workspaceId: string;
        changeSetId: string;
        kind: EntityKind;
        id: string;
      };
    }
  | {
      messageKind: "atomUpdated";
      arguments: {
        workspaceId: string;
        changeSetId: string;
        kind: EntityKind;
        id: string;
        data: AtomDocument;
        listIds: string[];
        removed: boolean;
      };
    }
  | {
      messageKind: "listenerInFlight";
      arguments: { changeSetId: ChangeSetId; label: string };
    }
  | {
      messageKind: "listenerReturned";
      arguments: { changeSetId: ChangeSetId; label: string };
    }
  | {
      messageKind: "lobbyExit";
      arguments: { workspaceId: string; changeSetId: string };
    };

export type Listable =
  | EntityKind.ViewComponentList
  | EntityKind.ComponentList
  | EntityKind.IncomingConnectionsList
  | EntityKind.ViewList;
export type Gettable = Exclude<EntityKind, Listable>;

export interface SharedDBInterface {
  initDB: (testing: boolean) => Promise<void>;
  migrate: (testing: boolean) => Promise<Database>;
  setBearer: (token: string) => void;
  initSocket(): Promise<void>;
  unregisterRemote(id: string): void;
  registerRemote(id: string, remote: Comlink.Remote<TabDBInterface>): void;
  broadcastMessage(message: BroadcastMessage): Promise<void>;
  setRemote(remoteId: string): Promise<void>;
  initBifrost(gotLockPort: MessagePort): Promise<void>;
  bifrostClose(): Promise<void>;
  bifrostReconnect(): Promise<void>;
  linkNewChangeset(
    workspaceId: string,
    headChangeSetId: string,
    changeSetId: string,
  ): Promise<void>;
  getPossibleConnections(
    workspaceId: string,
    changeSetId: string,
    destSchemaName: string,
    dest: Prop,
  ): Promise<CategorizedPossibleConnections>;
  getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<OutgoingConnections | undefined>;
  getOutgoingConnectionsCounts(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<Record<ComponentId, number>>;
  getComponentNames(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<Record<ComponentId, string>>;
  getSchemaMembers(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<string>;
  get(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: Gettable,
    id: Id,
  ): Promise<typeof NOROW | AtomDocument>;
  getList(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: Listable,
    id: Id,
  ): Promise<string>;
  mjolnir(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: EntityKind,
    id: Id,
    checksum?: Checksum,
  ): Promise<void>;

  changeSetExists(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<boolean>;
  niflheim(workspaceId: string, changeSetId: ChangeSetId): Promise<boolean>;
  exec(
    opts: ExecBaseOptions &
      ExecRowModeArrayOptions &
      (ExecReturnThisOptions | ExecReturnResultRowsOptions) & {
        sql: FlexibleString;
      },
  ): Promise<SqlValue[][]>;
  pruneAtomsForClosedChangeSet(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
  ): Promise<void>;
  bobby(): Promise<void>;
  ragnarok(
    workspaceId: string,
    changeSetId: string,
    noColdStart?: boolean,
  ): Promise<void>;
  // show me everything
  odin(changeSetId: ChangeSetId): Promise<object>;
}

export interface TabDBInterface {
  initDB: (testing: boolean) => Promise<void>;
  migrate: (testing: boolean) => Database;
  setBearer: (token: string) => void;
  initSocket(): Promise<void>;
  receiveBroadcast(message: BroadcastMessage): Promise<void>;
  setRemote(remoteId: string): Promise<void>;
  initBifrost(gotLockPort: MessagePort): Promise<void>;
  bifrostClose(): void;
  bifrostReconnect(): void;
  linkNewChangeset(
    workspaceId: string,
    headChangeSetId: string,
    changeSetId: string,
  ): void;
  getPossibleConnections(
    workspaceId: string,
    changeSetId: string,
    destSchemaName: string,
    dest: Prop,
  ): CategorizedPossibleConnections;
  getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): OutgoingConnections;
  getOutgoingConnectionsCounts(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Record<ComponentId, number>;
  getComponentNames(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Record<ComponentId, string>;
  getSchemaMembers(workspaceId: string, changeSetId: ChangeSetId): string;
  get(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: Gettable,
    id: Id,
  ): typeof NOROW | AtomDocument;
  getList(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: Listable,
    id: Id,
  ): string;
  mjolnirBulk(
    workspaceId: string,
    changeSetId: ChangeSetId,
    objs: MjolnirBulk,
    indexChecksum: string,
  ): Promise<void>;
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
  addAtomUpdated(fn: UpdateFn): void;
  atomChecksumsFor(
    changeSetId: ChangeSetId,
  ): Promise<Record<QueryKey, Checksum>>;
  changeSetExists(workspaceId: string, changeSetId: ChangeSetId): boolean;
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
  bobby(): void;
  ragnarok(
    workspaceId: string,
    changeSetId: string,
    noColdStart?: boolean,
  ): void;
  // show me everything
  odin(changeSetId: ChangeSetId): object;
}

export class Ragnarok extends Error {
  workspaceId: string;
  changeSetId: string;
  fromChecksumExpected: string | undefined;
  currentChecksum: string | undefined;

  constructor(
    message: string,
    workspaceId: string,
    changeSetId: string,
    fromChecksumExpected: string | undefined,
    currentChecksum: string | undefined,
  ) {
    super(message);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.fromChecksumExpected = fromChecksumExpected;
    this.currentChecksum = currentChecksum;
  }
}

export type RealSqlValue = NonNullable<SqlValue>;
export type RowWithColumns = Record<Column, SqlValue>;
export type RowID = Record<"id", number>;
export type RowWithColumnsAndId = RowID & RowWithColumns;
export type Records = (RowWithColumns | RowWithColumnsAndId)[];

export interface CategorizedPossibleConnections {
  suggestedMatches: Array<PossibleConnection>;
  typeAndNameMatches: Array<PossibleConnection>;
  typeMatches: Array<PossibleConnection>;
  nonMatches: Array<PossibleConnection>;
}

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
  fromIndexChecksum: Checksum;
  toIndexChecksum: Checksum;
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
  indexChecksum: string;
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
  indexChecksum: string;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type AtomDocument = any;
