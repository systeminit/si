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
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { DefaultMap } from "@/utils/defaultmap";
import { ComponentId } from "@/api/sdf/dal/component";
import { WorkspacePk } from "@/api/sdf/dal/workspace";
import { ViewId } from "@/api/sdf/dal/views";
import { Connection, DefaultSubscriptions, EntityKind, GlobalEntity, PossibleConnection } from "./entity_kind_types";

export type Column = string;
export type Columns = Column[];
export type BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  noBroadcast?: boolean,
) => void;

export type OutgoingConnections = DefaultMap<ComponentId, Record<string, Connection>>;

export type IncomingManagementConnections = DefaultMap<string, Record<string, Connection>>;

export type ConnStatusFn = (workspaceId: string, connected: boolean, noBroadcast?: boolean) => void;

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
export type RainbowFn = (changeSetId: ChangeSetId, label: string, noBroadcast?: boolean) => void;
export type LobbyExitFn = (workspacePk: string, changeSetId: string, noBroadcast?: boolean) => Promise<void>;

export type MjolnirBulk = Array<{
  kind: EntityKind;
  id: Id;
  checksum: Checksum;
}>;

export interface BulkSuccess {
  workspaceSnapshotAddress: string;
  frontEndObject: AtomWithData;
  indexChecksum: string;
}

export interface AtomWithDocument extends Common {
  doc: AtomDocument;
}

export const SHARED_BROADCAST_CHANNEL_NAME = "SHAREDWORKER_BROADCAST";
export const FORCE_LEADER_ELECTION = "FORCE_LEADER_ELECTION";
export const DB_NOT_INIT_ERR = "DB_NOT_INIT";

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
      messageKind: "updateConnectionStatus";
      arguments: { workspaceId: string; connected: boolean };
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
      messageKind: "interest";
      arguments: Record<string, number>;
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
export type Gettable = Exclude<Exclude<EntityKind, Listable>, GlobalEntity>;

export interface QueryAttributesTerm {
  key: string;
  value: string;
  op: "startsWith" | "exact";
}

export interface SharedDBInterface {
  initDB: (testing: boolean) => Promise<void>;
  migrate: (testing: boolean) => Promise<Database>;
  setBearer: (workspaceId: string, token: string) => Promise<void>;
  getBearers(): Promise<{ [key: string]: string }>;
  addBearers(bearers: { [key: string]: string }): Promise<void>;
  initSocket(workspaceId: string): Promise<void>;
  unregisterRemote(id: string): void;
  registerRemote(id: string, remote: Comlink.Remote<TabDBInterface>): Promise<void>;
  broadcastMessage(message: BroadcastMessage): Promise<void>;
  setLeader(remoteId: string): Promise<void>;
  hasLeader(): Promise<boolean>;
  currentLeaderId(): Promise<string | undefined>;
  initBifrost(gotLockPort: MessagePort): Promise<void>;
  bifrostClose(): Promise<void>;
  bifrostReconnect(): Promise<void>;
  linkNewChangeset(workspaceId: string, headChangeSetId: string, changeSetId: string): Promise<void>;
  getPossibleConnections(workspaceId: string, changeSetId: string): Promise<PossibleConnection[]>;
  getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<OutgoingConnections | undefined>;
  getIncomingManagementByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<IncomingManagementConnections | undefined>;
  getOutgoingConnectionsCounts(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ComponentId, number>>;
  getComponentDetails(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ComponentId, ComponentInfo>>;
  getComponentsInViews(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ViewId, Set<ComponentId>>>;
  getComponentsInOnlyOneView(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ComponentId, ViewId>>;
  getSchemaMembers(workspaceId: string, changeSetId: ChangeSetId): Promise<string>;
  getDefaultSubscriptions(workspaceId: string, changeSetId: ChangeSetId): Promise<DefaultSubscriptions>;
  getGlobal(workspaceId: string, kind: GlobalEntity, id: Id): Promise<-1 | AtomDocument>;
  get(workspaceId: string, changeSetId: ChangeSetId, kind: Gettable, id: Id): Promise<-1 | AtomDocument>;
  getExists(workspaceId: string, changeSetId: ChangeSetId, kind: Gettable, id: Id): Promise<boolean>;
  getList(workspaceId: string, changeSetId: ChangeSetId, kind: Listable, id: Id): Promise<string>;
  getKind(workspaceId: string, changeSetId: ChangeSetId, kind: EntityKind): Promise<string[]>;
  /**
   * Query AttributeTree MVs in a changeset, looking for components that match the given terms.
   *
   * @param workspaceId The workspace ID to query.
   * @param changeSetId The changeset ID to query.
   * @param terms The key/value pairs to match. e.g. { key: "vpcId", value: "vpc-123" } or { key: "/domain/vpcId", value: "vpc-123" }
   * @returns the list of component IDs that match the given terms.
   */
  queryAttributes(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
    terms: QueryAttributesTerm[],
  ): Promise<ComponentId[]>;
  mjolnir(workspaceId: string, changeSetId: ChangeSetId, kind: EntityKind, id: Id, checksum?: Checksum): Promise<void>;
  showInterest(workspaceId: string, changeSetId: ChangeSetId): Promise<void>;
  setConnections(connections: Record<string, boolean>): Promise<void>;
  getConnections(): Promise<Record<string, boolean>>;

  changeSetExists(workspaceId: string, changeSetId: ChangeSetId): Promise<boolean>;
  niflheim(workspaceId: string, changeSetId: ChangeSetId): Promise<-1 | 0 | 1>;
  syncAtoms(workspaceId: string, changeSetId: ChangeSetId): Promise<void>;
  vanaheim(workspaceId: string): Promise<boolean>;
  exec(
    opts: ExecBaseOptions &
      ExecRowModeArrayOptions &
      (ExecReturnThisOptions | ExecReturnResultRowsOptions) & {
        sql: FlexibleString;
      },
  ): Promise<SqlValue[][]>;
  pruneAtomsForClosedChangeSet(workspaceId: WorkspacePk, changeSetId: ChangeSetId): Promise<void>;
  bobby(): Promise<void>;
  ragnarok(workspaceId: string, changeSetId: string, noColdStart?: boolean): Promise<void>;
  // show me everything
  odin(changeSetId: ChangeSetId): Promise<object>;
}

export interface TabDBInterface {
  initDB: (testing: boolean) => Promise<void>;
  hasDbLock(): Promise<boolean>;
  createLock(): void;
  migrate: (testing: boolean) => Database;
  setBearer: (workspaceId: string, token: string) => void;
  initSocket(workspaceId: string): Promise<void>;
  receiveInterest(interest: Record<string, number>): void;
  receiveBroadcast(message: BroadcastMessage): Promise<void>;
  initBifrost(gotLockPort: MessagePort, userPk: string): Promise<string>;
  bifrostClose(): void;
  bifrostReconnect(): void;
  linkNewChangeset(workspaceId: string, headChangeSetId: string, changeSetId: string): Promise<void>;
  getPossibleConnections(workspaceId: string, changeSetId: string): PossibleConnection[];
  getOutgoingConnectionsByComponentId(workspaceId: string, changeSetId: ChangeSetId): OutgoingConnections;
  getIncomingManagementByComponentId(workspaceId: string, changeSetId: ChangeSetId): IncomingManagementConnections;
  getOutgoingConnectionsCounts(workspaceId: string, changeSetId: ChangeSetId): Record<ComponentId, number>;
  getComponentDetails(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ComponentId, ComponentInfo>>;
  getComponentsInViews(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ViewId, Set<ComponentId>>>;
  getComponentsInOnlyOneView(workspaceId: string, changeSetId: ChangeSetId): Promise<Record<ComponentId, ViewId>>;
  getSchemaMembers(workspaceId: string, changeSetId: ChangeSetId): Promise<string>;
  getDefaultSubscriptions(workspaceId: string, changeSetId: ChangeSetId): DefaultSubscriptions;
  getGlobal(workspaceId: string, kind: GlobalEntity, id: Id): Promise<-1 | AtomDocument>;
  get(workspaceId: string, changeSetId: ChangeSetId, kind: Gettable, id: Id): Promise<-1 | AtomDocument>;
  getExists(workspaceId: string, changeSetId: ChangeSetId, kind: Gettable, id: Id): Promise<boolean>;
  getKind(workspaceId: string, changeSetId: ChangeSetId, kind: EntityKind): Promise<string[]>;
  getList(workspaceId: string, changeSetId: ChangeSetId, kind: Listable, id: Id): Promise<string>;
  /**
   * Query AttributeTree MVs in a changeset, looking for components that match the given terms.
   *
   * @param workspaceId The workspace ID to query.
   * @param changeSetId The changeset ID to query.
   * @param terms The key/value pairs to match. e.g. { key: "vpcId", value: "vpc-123" } or { key: "/domain/vpcId", value: "vpc-123" }
   * @returns the list of component IDs that match the given terms.
   */
  queryAttributes(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
    terms: QueryAttributesTerm[],
  ): Promise<ComponentId[]>;
  mjolnirBulk(workspaceId: string, changeSetId: ChangeSetId, objs: MjolnirBulk, indexChecksum: string): Promise<void>;
  mjolnir(workspaceId: string, changeSetId: ChangeSetId, kind: EntityKind, id: Id, checksum?: Checksum): void;
  partialKeyFromKindAndId(kind: EntityKind, id: Id): QueryKey;
  kindAndIdFromKey(key: QueryKey): { kind: EntityKind; id: Id };
  addListenerBustCache(fn: BustCacheFn): void;
  addListenerInFlight(fn: RainbowFn): void;
  addListenerReturned(fn: RainbowFn): void;
  addListenerLobbyExit(fn: LobbyExitFn): void;
  addAtomUpdated(fn: UpdateFn): void;
  addConnStatusFn(fn: ConnStatusFn): void;
  changeSetExists(workspaceId: string, changeSetId: ChangeSetId): Promise<boolean>;
  niflheim(workspaceId: string, changeSetId: ChangeSetId): Promise<-1 | 0 | 1>;
  syncAtoms(workspaceId: string, changeSetId: ChangeSetId): Promise<void>;
  vanaheim(workspaceId: string): Promise<boolean>;
  pruneAtomsForClosedChangeSet(workspaceId: WorkspacePk, changeSetId: ChangeSetId): void;
  /* these are used for testing purposes, and should not be used outside the web worker in production code */
  oneInOne(rows: SqlValue[][]): SqlValue | typeof NOROW;
  bulkCreateAtoms(indexObjects: (BulkSuccess | AtomWithDocument)[], chunkSize?: number): void;
  bulkInsertAtomMTMs(
    indexObjects: (BulkSuccess | AtomWithDocument)[],
    indexChecksum: Checksum,
    chunkSize?: number,
  ): void;
  encodeDocumentForDB(doc: object): Uint8Array;
  decodeDocumentFromDB(doc: ArrayBuffer): AtomDocument;
  handleDeploymentPatchMessage(data: DeploymentPatchBatch): Promise<void>;
  handleWorkspacePatchMessage(data: WorkspacePatchBatch): Promise<void>;
  handleIndexMvPatch(data: WorkspaceIndexUpdate): Promise<void>;
  handleHammer(msg: WorkspaceAtomMessage): Promise<void>;
  exec(
    opts: ExecBaseOptions &
      ExecRowModeArrayOptions &
      (ExecReturnThisOptions | ExecReturnResultRowsOptions) & {
        sql: FlexibleString;
      },
  ): SqlValue[][];
  bobby(): void;
  ragnarok(workspaceId: string, changeSetId: string, noColdStart?: boolean): void;
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
    else results.push(row);
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

export interface DeploymentAtomMeta {
  fromIndexChecksum: Checksum;
  toIndexChecksum: Checksum;
}

export interface WorkspaceAtomMeta extends DeploymentAtomMeta {
  workspaceId: WorkspacePk;
  changeSetId: ChangeSetId;
}

interface Atom extends AbstractAtom {
  operations?: Operation[];
}

export type WorkspaceAtom = Atom & WorkspaceAtomMeta;
export type DeploymentAtom = Atom & DeploymentAtomMeta;

export enum MessageKind {
  WORKSPACE_PATCH = "PatchMessage",
  DEPLOYMENT_PATCH = "DeploymentPatchMessage",
  MJOLNIR = "MjolnirAtom",
  WORKSPACE_INDEXUPDATE = "IndexUpdate",
  DEPLOYMENT_INDEXUPDATE = "DeploymentIndexUpdate",
}

export interface WorkspaceAtomMessage {
  kind: MessageKind.MJOLNIR;
  atom: WorkspaceAtom;
  data: object;
}

export interface WorkspacePatchBatch {
  meta: WorkspaceAtomMeta;
  kind: MessageKind.WORKSPACE_PATCH;
  patches: AtomOperation[];
}

export interface DeploymentPatchBatch {
  meta: DeploymentAtomMeta;
  kind: MessageKind.DEPLOYMENT_PATCH;
  patches: AtomOperation[];
}

export interface WorkspaceIndexUpdate {
  meta: WorkspaceAtomMeta;
  kind: MessageKind.WORKSPACE_INDEXUPDATE;
  indexChecksum: string;
  patch: null | AtomOperation;
}

export interface DeploymentIndexUpdate {
  meta: DeploymentAtomMeta;
  kind: MessageKind.DEPLOYMENT_INDEXUPDATE;
  indexChecksum: string;
}

export interface Common {
  kind: EntityKind;
  id: Id;
  checksum: string;
}

export interface StoredMvIndex {
  mvList: Common[];
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

export interface AtomWithData extends Common {
  data: object;
}

export type AtomDocument = object & { id: string };

export type ComponentInfo = {
  name: string;
  schemaVariantName: string;
};
