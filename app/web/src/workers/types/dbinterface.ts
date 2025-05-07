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
import { Categories } from "@/store/components.store";
import { ActionProposedView } from "@/store/actions.store";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  InputSocketId,
  OutputSocketId,
  SchemaId,
  SchemaVariantId,
} from "@/api/sdf/dal/schema";
import { ActionKind, ActionPrototypeId } from "@/api/sdf/dal/action";
import { FuncId } from "@/api/sdf/dal/func";
import { AttributeValueId } from "@/store/status.store";
import { PropId, PropKind } from "@/api/sdf/dal/prop";
import {
  PropertyEditorPropWidgetKind,
  ValidationOutput,
} from "@/api/sdf/dal/property_editor";
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
  initDB: (testing: boolean) => Promise<void>;
  migrate: (testing: boolean) => void;
  setBearer: (token: string) => void;
  initSocket(): Promise<void>;
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
  ragnarok(workspaceId: string, changeSetId: string): Promise<void>;
  // show me everything
  odin(changeSetId: ChangeSetId): object;
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

// ==========================================================================
// FAKING IT - CORE TYPES
interface Reference {
  id: string;
  checksum: string;
  kind: string;
}

interface WeakReference {
  id: string;
  kind: string;
}

// ==========================================================================
// FAKING IT - EVERYTHING ELSE
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

export interface RawViewList {
  id: string;
  views: Reference[];
}

export interface BifrostSchemaVariantCategories {
  id: string; // change set id
  categories: Categories;
}

export interface BifrostActionViewList {
  id: ChangeSetId;
  actions: ActionProposedView[];
}

export interface ComponentQualificationTotals {
  total: number;
  warned: number;
  succeeded: number;
  failed: number;
  running: number;
}

export interface BifrostComponent {
  id: ComponentId;
  name: string;
  color?: string;
  schemaName: string;
  schemaId: SchemaId;
  schemaVariantId: SchemaVariantId;
  schemaVariantName: string;
  schemaVariantDescription?: string;
  schemaVariantDocLink?: string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  inputCount: number;
  diffCount: number;
  rootAttributeValueId: AttributeValueId;
  domainAttributeValueId: AttributeValueId;
  secretsAttributeValueId: AttributeValueId;
  siAttributeValueId: AttributeValueId;
  resourceValueAttributeValueId: AttributeValueId;
  resourceDiff: {
    current?: string;
    diff?: string;
  };
}

export interface BifrostComponentList {
  id: ChangeSetId;
  components: BifrostComponent[];
}

export interface ViewComponentList {
  id: string; // ViewId
  components: BifrostComponent[];
}

export interface RawComponentList {
  id: ChangeSetId;
  components: Reference[];
}

export interface ActionPrototypeView {
  id: ActionPrototypeId;
  funcId: FuncId;
  kind: ActionKind;
  displayName?: string;
  name: string;
}

export interface BifrostActionPrototypeViewList {
  id: SchemaVariantId;
  actionPrototypes: ActionPrototypeView[];
}

export interface Prop {
  id: PropId;
  path: string;
  name: string;
  kind: PropKind;
  widgetKind: PropertyEditorPropWidgetKind;
  docLink?: string;
  documentation?: string;
  validationFormation?: string;
  defaultCanBeSetBySocket: boolean;
  isOriginSecret: boolean;
  createOnly: boolean;
}

export interface AttributeValue {
  id: AttributeValueId;
  key?: string;
  path?: string;
  propId?: string;
  value: string;
  canBeSetBySocket: boolean;
  isFromExternalSource: boolean;
  isControlledByAncestor: boolean;
  isControlledByDynamicFunc: boolean;
  overriden: boolean;
  validation?: ValidationOutput;
}

export interface AVTree {
  parent?: AttributeValueId;
  children: AttributeValueId[];
}

export interface BifrostAttributeTree {
  id: ComponentId;
  attributeValues: Record<AttributeValueId, AttributeValue>;
  props: Record<PropId, Prop>;
  treeInfo: Record<AttributeValueId, AVTree>;
}

export interface RawIncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: Reference[];
}

export interface BifrostIncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: BifrostIncomingConnections[];
}

export interface RawIncomingConnections {
  id: ComponentId;
  connections: RawConnection[];
}

export interface BifrostIncomingConnections {
  id: ComponentId;
  component: BifrostComponent;
  connections: Connection[];
}

type RawConnection =
  | {
      kind: "prop";
      fromComponentId: WeakReference; // this is why we need the raw object
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromPropId: PropId;
      fromPropPath: string;
      toComponentId: WeakReference; // this is why we need the raw object
      toPropId: PropId;
      toPropPath: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    }
  | {
      kind: "socket";
      fromComponentId: WeakReference; // this is why we need the raw object
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromSocketId: OutputSocketId;
      fromSocketName: string;
      toComponentId: WeakReference; // this is why we need the raw object
      toSocketId: InputSocketId;
      toSocketName: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    };

type Connection =
  | {
      kind: "prop";
      fromComponent: BifrostComponent; // we will have the full component from the weak reference
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromPropId: PropId;
      fromPropPath: string;
      toComponent: BifrostComponent; // we will have the full component from the weak reference
      toPropId: PropId;
      toPropPath: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    }
  | {
      kind: "socket";
      fromComponent: BifrostComponent; // we will have the full component from the weak reference
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromSocketId: OutputSocketId;
      fromSocketName: string;
      toComponent: BifrostComponent; // we will have the full component from the weak reference
      toSocketId: InputSocketId;
      toSocketName: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    };
