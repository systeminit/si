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
import { ActionProposedView } from "@/store/actions.store";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  InputSocket,
  InputSocketId,
  OutputSocket,
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
import { ViewId } from "@/api/sdf/dal/views";
import { DefaultMap } from "@/utils/defaultmap";

export type Column = string;
export type Columns = Column[];
export type BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
) => void;

export type OutgoingConnections = DefaultMap<string, BifrostConnection[]>;
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

export enum EntityKind {
  Component = "Component",
  ViewList = "ViewList",
  ComponentList = "ComponentList",
  ViewComponentList = "ViewComponentList",
  IncomingConnections = "IncomingConnections",
  IncomingConnectionsList = "IncomingConnectionsList",
  SchemaVariantCategories = "SchemaVariantCategories",
  ActionViewList = "ActionViewList",
  ActionPrototypeViewList = "ActionPrototypeViewList",
  SecretsList = "SecretsList",
  SecretDefinitionList = "SecretDefinitionList",
  Secret = "Secret",
  SecretDefinition = "SecretDefinitionList",
  OutgoingConnections = "OutgoingConnections",
  PossibleConnections = "PossibleConnections",
}
/**
 * NOTE, if you want to narrow the type of a variable
 * that maybe an entity kind, use this fn
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const isEntityKind = (maybeEntityKind: any): EntityKind | null => {
  const k = maybeEntityKind as string; // first cast to string, since enum values are strings
  // if the string-y value is in the enum
  if (k in EntityKind)
    // you can safely cast this
    return k as EntityKind;
  return null;
};

interface Reference {
  id: string;
  checksum: string;
  kind: EntityKind;
}

interface WeakReference {
  id: string;
  kind: EntityKind;
}

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

/**
 * NAMING RULES
 * 1. If the type is not mutated at all
 * (e.g. it does not have an if-block in `getReferences` or `getComputed`)
 * THOU SHALT name it according the entity kind--EXACT MATCH PLEASE
 *
 * 2. If the type is mutated, you will prefix the EXACT entity kind with
 * THOU SHALT prefix the type that comes over the wire with `Edda`
 * THOU SHALT prefix the type that returns from `bifrost` fn with `Bifrost`
 *
 * 3. `EddaXXX` types SHALL NOT have `BifrostXXX` types on them
 * 4. `BifrostXXX` types SHALL NOT have `EddaXXX` types on them
 * 5. Vue components SHALL NEVER use `EddaXXX` types
 * 6. `getReferences` SHALL NEVER return an `EddaXXX` type
 *    (e.g. perform the full translation from Edda to Bifrost)
 * 7. `getReferences` SHALL set default/warning data that `getComputed` will write over
 */
export type PossibleConnection = {
  attributeValueId: string;
  name: string;
  path: string;
  value: string;
  componentName: string;
  schemaName: string;
  componentId: string;
  annotation: string;
};

export interface View {
  id: string;
  name: string;
  isDefault: boolean;
  created_at: string;
  updated_at: string;
}

export interface BifrostViewList {
  id: string;
  views: View[];
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

export interface EddaComponent {
  id: ComponentId;
  name: string;
  color?: string;
  schemaName: string;
  schemaId: SchemaId;
  schemaVariantId: Reference;
  schemaVariantName: string;
  schemaVariantDescription?: string;
  schemaVariantDocLink?: string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  inputCount: number;
  // this will only be filled in when it is computed
  outputCount: number;
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
  attributeTree: AttributeTree;
}

export interface UninstalledVariant {
  schemaId: string;
  schemaName: string;
  displayName: string | null;
  category: string;
  color: string;
  link: string | null;
  description: string | null;
}

export interface CategoryInstalledVariant {
  type: "installed";
  id: string;
  variant: SchemaVariant;
}

export interface CategoryUninstalledVariant {
  type: "uninstalled";
  id: string;
  variant: UninstalledVariant;
}

export type CategoryVariant =
  | CategoryInstalledVariant
  | CategoryUninstalledVariant;

export type Categories = {
  displayName: string;
  schemaVariants: CategoryVariant[];
}[];

export interface PropTree {
  props: Record<PropId, Prop>;
  treeInfo: Record<PropId, { parent?: PropId; children: PropId[] }>;
}
export interface SchemaVariant {
  id: string;
  schemaVariantId: string;
  schemaName: string;
  schemaDocLinks?: string;
  displayName: string | null;
  category: string;
  color: string;
  link: string | null;
  description: string | null;

  created_at: IsoDateString;
  updated_at: IsoDateString;

  version: string;
  isLocked: boolean;

  schemaId: SchemaId;

  inputSockets: InputSocket[];
  outputSockets: OutputSocket[];
  propTree: PropTree;
  canCreateNewComponents: boolean;

  canContribute: boolean;
  mgmtFunctions: {
    id: string;
    funcId: FuncId;
    description?: string;
    prototypeName: string;
    name: string;
    kind: string;
  }[];
}

export interface BifrostComponent {
  id: ComponentId;
  name: string;
  color?: string;
  schemaName: string;
  schemaId: SchemaId;
  schemaVariant: SchemaVariant;
  schemaVariantName: string;
  schemaVariantDescription?: string;
  schemaVariantDocLink?: string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  inputCount: number;
  // this will only be filled in when it is computed
  outputCount: number;
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
  attributeTree: AttributeTree;
}

export interface BifrostComponentList {
  id: ChangeSetId;
  components: BifrostComponent[];
}

export interface ViewComponentList {
  id: ViewId;
  components: BifrostComponent[];
}

export interface EddaComponentList {
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

export interface ActionPrototypeViewList {
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
  // this is for output sources
  eligibleToReceiveData: boolean;
  // this is for input sources
  eligibleToSendData: boolean;
  hidden: boolean;
}

export interface AttributeValue {
  id: AttributeValueId;
  key?: string;
  path?: string;
  propId?: string;
  value: string | null;
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

export interface AttributeTree {
  attributeValues: Record<AttributeValueId, AttributeValue>;
  props: Record<PropId, Prop>;
  treeInfo: Record<AttributeValueId, AVTree>;
}

export interface EddaIncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: Reference[];
}

export interface BifrostIncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: BifrostComponentConnections[];
}

export interface EddaIncomingConnections {
  id: ComponentId;
  connections: EddaConnection[];
}

export interface BifrostComponentConnections {
  id: ComponentId;
  component: BifrostComponent;
  incoming: BifrostConnection[];
  // note: outgoing connections cannot be computed right now
}

export type EddaConnection =
  | {
      kind: "prop";
      fromComponentId: WeakReference;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromPropId: PropId;
      fromPropPath: string;
      toComponentId: WeakReference;
      toPropId: PropId;
      toPropPath: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    }
  | {
      kind: "socket";
      fromComponentId: WeakReference;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromSocketId: OutputSocketId;
      fromSocketName: string;
      toComponentId: WeakReference;
      toSocketId: InputSocketId;
      toSocketName: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    };

export type BifrostConnection =
  | {
      kind: "prop";
      fromComponent: BifrostComponent;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromPropId: PropId;
      fromPropPath: string;
      toComponent: BifrostComponent;
      toPropId: PropId;
      toPropPath: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    }
  | {
      kind: "socket";
      fromComponent: BifrostComponent;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromSocketId: OutputSocketId;
      fromSocketName: string;
      toComponent: BifrostComponent;
      toSocketId: InputSocketId;
      toSocketName: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    };
