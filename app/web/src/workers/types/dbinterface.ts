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
import {
  Connection,
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
  noBroadcast?: boolean,
) => void;

export type OutgoingConnections = DefaultMap<
  string,
  Record<string, Connection>
>;

export type IncomingManagementConnections = DefaultMap<
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
  registerRemote(
    id: string,
    remote: Comlink.Remote<TabDBInterface>,
  ): Promise<void>;
  broadcastMessage(message: BroadcastMessage): Promise<void>;
  setLeader(remoteId: string): Promise<void>;
  hasLeader(): Promise<boolean>;
  currentLeaderId(): Promise<string | undefined>;
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
  ): Promise<PossibleConnection[]>;
  getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<OutgoingConnections | undefined>;
  getIncomingManagementByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<IncomingManagementConnections | undefined>;
  getOutgoingConnectionsCounts(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<Record<ComponentId, number>>;
  getComponentDetails(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<Record<ComponentId, ComponentInfo>>;
  getComponentsInViews(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<Record<ViewId, Set<ComponentId>>>;
  getComponentsInOnlyOneView(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Promise<Record<ComponentId, ViewId>>;
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
  hasDbLock(): Promise<boolean>;
  migrate: (testing: boolean) => Database;
  setBearer: (workspaceId: string, token: string) => void;
  initSocket(workspaceId: string): Promise<void>;
  receiveBroadcast(message: BroadcastMessage): Promise<void>;
  initBifrost(gotLockPort: MessagePort): Promise<string>;
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
  ): PossibleConnection[];
  getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): OutgoingConnections;
  getIncomingManagementByComponentId(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): IncomingManagementConnections;
  getOutgoingConnectionsCounts(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Record<ComponentId, number>;
  getComponentDetails(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Record<ComponentId, ComponentInfo>;
  getComponentsInViews(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Record<ViewId, Set<ComponentId>>;
  getComponentsInOnlyOneView(
    workspaceId: string,
    changeSetId: ChangeSetId,
  ): Record<ComponentId, ViewId>;
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
  ): ComponentId[];
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
  handleWorkspacePatchMessage(data: WorkspacePatchBatch): Promise<void>;
  handleHammer(msg: WorkspaceAtomMessage): Promise<void>;
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

export type ComponentInfo = {
  name: string;
  schemaVariantName: string;
};
