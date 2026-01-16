import { ActionProposedView } from "@/store/actions.store";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { SchemaId, SchemaVariantId } from "@/api/sdf/dal/schema";
import { ActionId, ActionKind, ActionPrototypeId, ActionState } from "@/api/sdf/dal/action";
import { FuncId } from "@/api/sdf/dal/func";
import { AttributeValueId } from "@/store/status.store";
import { PropId, PropKind } from "@/api/sdf/dal/prop";
import { PropertyEditorPropWidgetKind, ValidationOutput } from "@/api/sdf/dal/property_editor";
import { ViewId } from "@/api/sdf/dal/views";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { DefaultMap } from "@/utils/defaultmap";
import { ComponentName } from "@/store/components.store";
import { WorkspacePk } from "@/newhotness/types";
import { SecretId } from "@/store/realtime/realtime_events";
import { ComponentInfo } from "./dbinterface";

export enum EntityKind {
  ActionDiffList = "ActionDiffList",
  ActionPrototypeViewList = "ActionPrototypeViewList",
  ActionViewList = "ActionViewList",
  AttributeTree = "AttributeTree",
  AuditLogsForComponent = "AuditLogsForComponent",
  Component = "Component",
  ComponentDiff = "ComponentDiff",
  ComponentDetails = "ComponentDetails",
  ComponentInList = "ComponentInList",
  ComponentList = "ComponentList",
  ComponentsInOnlyOneView = "ComponentsInOnlyOneView",
  ComponentsInViews = "ComponentsInViews",
  DefaultSubscriptions = "DefaultSubscriptions",
  DependentValueComponentList = "DependentValueComponentList",
  DependentValues = "DependentValues",
  ErasedComponents = "ErasedComponents",
  IncomingConnections = "IncomingConnections",
  IncomingConnectionsList = "IncomingConnectionsList",
  IncomingManagementConnections = "IncomingManagementConnections",
  ManagementConnections = "ManagementConnections",
  OutgoingConnections = "OutgoingConnections",
  OutgoingCounts = "OutgoingCounts",
  PossibleConnections = "PossibleConnections",
  QueryAttributes = "QueryAttributes",
  SchemaMembers = "SchemaMembers",
  SchemaVariant = "SchemaVariant",
  SecretDefinition = "SecretDefinition",
  View = "View",
  ViewComponentList = "ViewComponentList",
  ViewList = "ViewList",
  // The IndexMv itself
  MvIndex = "ChangeSetMvIndex",
  // IGNORING THESE
  LuminorkDefaultVariant = "LuminorkDefaultVariant",
  LuminorkSchemaVariant = "LuminorkSchemaVariant",
  // DEPLOYMENT aka GLOBAL
  CachedSchema = "CachedSchema",
  CachedSchemaVariant = "CachedSchemaVariant",
  CachedDefaultVariant = "CachedDefaultVariant",
}

export const GLOBAL_ENTITIES = [EntityKind.CachedSchema, EntityKind.CachedDefaultVariant] as const;
export type GlobalEntity = (typeof GLOBAL_ENTITIES)[number];

export const GLOBAL_IDENTIFIER = "-";

/**
 * NOTE, if you want to narrow the type of a variable
 * that maybe an entity kind, use this fn
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const isEntityKind = (maybeEntityKind: any): EntityKind | null => {
  const k = maybeEntityKind as string; // first cast to string, since enum values are strings
  // if the string-y value is in the enum
  if (k in EntityKind) {
    // you can safely cast this
    return k as EntityKind;
  }
  return null;
};

interface Reference<T extends EntityKind> {
  id: string;
  checksum: string;
  kind: T;
}

interface WeakReference<T extends EntityKind> {
  id: string;
  kind: T;
}

export type OutgoingConnections = DefaultMap<string, Connection[]>;

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
export type OutgoingCounts = Record<ComponentId, number>;
export type ComponentDetails = Record<ComponentId, ComponentInfo>;

export type PossibleConnection = {
  attributeValueId: string;
  name: string;
  path: string;
  value: JsonValue;
  componentName: string;
  schemaName: string;
  componentId: string;
  kind: string;
  isOriginSecret: boolean;
  suggestAsSourceFor?: null | PropSuggestion[];
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
  views: Reference<EntityKind.View>[];
}

export interface BifrostActionViewList {
  id: ChangeSetId;
  actions: ActionProposedView[];
}

export interface ActionDiffList {
  id: WorkspacePk;
  actionDiffs: Record<ActionId, ActionDiffView>;
}

export interface ActionDiffView {
  id: ActionId;
  componentId: ComponentId;
  diffStatus: ActionDiffStatus;
}

export type ActionDiffStatus =
  | {
      Added: { new_state: ActionState };
    }
  | {
      Modified: { old_state: ActionState; new_state: ActionState };
    }
  | "None"
  | "Removed";

export interface ComponentQualificationTotals {
  total: number;
  warned: number;
  succeeded: number;
  failed: number;
}

export interface EddaComponent {
  id: ComponentId;
  name: string;
  color?: null | string;
  schemaName: string;
  schemaId: SchemaId;
  schemaVariantId: WeakReference<EntityKind.SchemaVariant>;
  schemaVariantName: string;
  schemaVariantDescription?: null | string;
  schemaVariantDocLink?: null | string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  isSecretDefining: boolean;
  inputCount: number;
  // this will only be filled in when it is computed
  outputCount: number;
  rootAttributeValueId: AttributeValueId;
  domainAttributeValueId: AttributeValueId;
  secretsAttributeValueId: AttributeValueId;
  siAttributeValueId: AttributeValueId;
  resourceValueAttributeValueId: AttributeValueId;
  resourceDiff: {
    current?: null | string;
    diff?: null | string;
  };
  attributeTree: AttributeTree;
  schemaMembers: WeakReference<EntityKind.SchemaMembers>;
  toDelete: boolean;
}

export interface SchemaMembers {
  id: SchemaId;
  defaultVariantId: SchemaVariantId;
  editingVariantId?: null | SchemaVariantId;
}

export interface UninstalledVariant {
  schemaId: SchemaId;
  schemaName: string;
  displayName: string | null;
  category: string;
  color: string;
  link: string | null;
  description: string | null;
  uninstalled: "uninstalled";
  isLocked: boolean;
}

export type CategoryVariant = SchemaVariant | UninstalledVariant | CachedDefaultVariant;

export type Categories = {
  displayName: string;
  schemaVariants: CategoryVariant[];
}[];

export interface PropTree {
  props: Record<PropId, Prop>;
  treeInfo: Record<PropId, { parent?: null | PropId; children: PropId[] }>;
}

export interface MgmtFunction {
  id: string;
  funcId: FuncId;
  description?: null | string;
  prototypeName: string;
  name: string;
  kind: MgmtFuncKind;
}

export enum MgmtFuncKind {
  Discover = "discover",
  Import = "import",
  Other = "other",
  RunTemplate = "runTemplate",
}

export interface SchemaVariant {
  id: string;
  schemaVariantId: string;
  schemaName: string;
  schemaDocLinks?: null | string;
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

  isSecretDefining: boolean;
  propTree: PropTree;
  canCreateNewComponents: boolean;

  canContribute: boolean;
  mgmtFunctions: MgmtFunction[];
}

export interface BifrostComponent {
  id: ComponentId;
  name: string;
  color?: null | string;
  schemaName: string;
  schemaId: SchemaId;
  // Needed for "ComponentInList" usage where the "SchemaVariant" is dropped.
  schemaVariantId: WeakReference<EntityKind.SchemaVariant>;
  schemaVariant: SchemaVariant;
  schemaVariantName: string;
  schemaVariantDescription?: null | string;
  schemaVariantDocLink?: null | string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  isSecretDefining: boolean;
  canBeUpgraded: boolean;
  inputCount: number;
  // this will only be filled in when it is computed
  outputCount: number;
  rootAttributeValueId: AttributeValueId;
  domainAttributeValueId: AttributeValueId;
  secretsAttributeValueId: AttributeValueId;
  siAttributeValueId: AttributeValueId;
  resourceValueAttributeValueId: AttributeValueId;
  resourceDiff: {
    current?: null | string;
    diff?: null | string;
  };
  toDelete: boolean;
}

export type ComponentDiffStatus = "Added" | "None" | "Modified" | "Removed";

export interface ComponentDiff {
  id: ComponentId;
  diffStatus: ComponentDiffStatus;
  attributeDiffs: Record<AttributePath, AttributeDiff>;
  resourceDiff: {
    current?: null | string;
    diff?: null | string;
  };
}

/**
 * The diff of the attribute.
 *
 * - If it is Modified, both "old" and "new" values will exist.
 * - If it is Added, only the "new" value will exist.
 * - If it is Removed, only the "old" value will exist.
 */
export type AttributeDiff =
  | {
      // Modified
      /** The current value of the attribute */
      new: AttributeSourceAndValue;
      /** The previous value of the attribute */
      old: AttributeSourceAndValue;
    }
  | {
      // Added
      new: AttributeSourceAndValue;
      old?: null | undefined;
    }
  | {
      // Removed
      new?: null | undefined;
      old: AttributeSourceAndValue;
    };

/**
 * An attribute's source, as well as its current value.
 *
 * - Subscription:
 *
 *       {
 *            $value: "us-east-1",
 *            $source: {
 *                component: "My Region",
 *                path: "/domain/region"
 *            }
 *       }
 *
 * - Value:
 *
 *       {
 *           $value: "us-east-1",
 *           $source: {
 *               component: "My Region",
 *               path: "/domain/region"
 *           }
 *       }
 *
 * - Value:
 *
 *       {
 *           $value: "us-east-1",
 *           $source: {
 *               value: "us-east-1"
 *           }
 *       }
 * - Set by a parent subscription:
 *
 *       {
 *          $value: "127.0.0.1"
 *          $source: {
 *              fromAncestor: "/domain/SecurityGroupIngress/3",
 *              component: "My Security Group Ingress Rule",
 *              path: "/domain",
 *          }
 *       }
 *
 * - Set by schema (e.g. attribute function):
 *
 *       {
 *           $value: "ami-1234567890EXAMPLE",
 *           $source: {
 *               fromSchema: true,
 *               prototype: "AWS_EC2_AMI:getImageIdFromAws()"
 *           }
 *       }
 *
 *       {
 *           $value: "Region is us-east-2",
 *           $source: {
 *               fromSchema: true,
 *               fromAncestor: "/domain/Rendered",
 *               prototype: "String_Template:renderValue()"
 *           }
 *       }
 */
export interface AttributeSourceAndValue {
  /**
   * Where the value comes from (the function, subscription, value, whether it comes from the
   * schema, etc.).
   */
  $source: AttributeSourceLocation & SimplifiedAttributeSource;

  /**
   * The current value.
   *
   * If it's a subscription or dynamic function, this will change when the value changes, even
   * though the source does not.
   */
  $value?: null | unknown;

  /**
   * If the source is pointed at a secret, this has name and other information about it.
   */
  $secret?: null | Secret;
}

/** The place in the tree this came from */
export interface AttributeSourceLocation {
  /** true if the value came from a "default" or attribute function on the schema */
  fromSchema?: null | true;
  /** If this came from a dynamic function on a *parent* attribute, this is the path to that attribute */
  fromAncestor?: null | AttributePath;
}

export type SimplifiedAttributeSource =
  | {
      component: ComponentId;
      componentName: ComponentName;
      path: AttributePath;

      prototype?: null | undefined;
      value?: null | undefined;
    }
  | {
      value: unknown;

      component?: null | undefined;
      componentName?: null | undefined;
      path?: null | undefined;
      prototype?: null | undefined;
    }
  | {
      prototype: string;

      component?: null | undefined;
      componentName?: null | undefined;
      path?: null | undefined;
      value?: null | undefined;
    };

export interface ErasedComponents {
  id: string;
  erased: Record<
    ComponentId,
    {
      diff: ComponentDiff;
      component: ComponentInList;
      resourceDiff: {
        current?: null | string;
        diff?: null | string;
      };
    }
  >;
}

// NOTE: when using `getMany` you don't end up with a BifrostComponent (b/c it doesnt have SchemaVariant)
// You end up with a ComponentInList
export interface ComponentInList {
  id: ComponentId;
  name: string;
  color?: null | string;
  schemaName: string;
  schemaId: SchemaId;
  // Needed for "ComponentInList" usage where the "SchemaVariant" is dropped.
  schemaVariantId: SchemaVariantId;
  schemaVariantName: string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  inputCount: number;
  diffStatus: ComponentDiffStatus;
  toDelete: boolean;
  resourceId: string | null;
  hasSocketConnections: boolean;
}

export interface BifrostComponentList {
  id: ChangeSetId;
  components: ComponentInList[];
}

export interface ViewComponentList {
  id: ViewId;
  components: ComponentInList[];
}

export interface EddaComponentList {
  id: ChangeSetId;
  components: WeakReference<EntityKind.Component>[];
}

export interface ActionPrototypeView {
  id: ActionPrototypeId;
  funcId: FuncId;
  kind: ActionKind;
  displayName?: null | string;
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
  childKind?: null | PropKind;
  widgetKind?: null | PropertyEditorPropWidgetKind;
  docLink?: null | string;
  documentation?: null | string;
  validationFormat?: null | string;
  defaultCanBeSetBySocket: boolean;
  isOriginSecret: boolean;
  secretDefinition?: null | SecretDefinition;
  createOnly: boolean;
  eligibleForConnection: boolean;
  hidden: boolean;
  suggestSources?: null | PropSuggestion[];
  suggestAsSourceFor?: null | PropSuggestion[];
}

export interface PropSuggestion {
  schema: string;
  prop: string;
}

export type JsonValue =
  | string
  | number
  | boolean
  | { [key: string]: JsonValue } // Object
  | JsonValue[] // Array
  | null;

export interface AttributeValue {
  id: AttributeValueId;
  key?: null | string;
  path: AttributePath;
  propId?: null | PropId;
  value: JsonValue;
  canBeSetBySocket?: null | boolean;
  externalSources?: null | ExternalSource[];
  isControlledByAncestor: boolean;
  isControlledByDynamicFunc: boolean;
  overridden: boolean;
  validation?: null | ValidationOutput;
  secret: Secret | null;
  hasSocketConnection: boolean;
  isDefaultSource: boolean;
}

export interface ExternalSource {
  path: string;
  componentId: ComponentId;
  componentName: string;
  isSecret: boolean;
}

export interface AVTree {
  parent?: null | AttributeValueId;
  children: AttributeValueId[];
}

export interface AttributeTree {
  id: ComponentId;
  attributeValues: Record<AttributeValueId, AttributeValue>;
  props: Record<PropId, Prop>;
  treeInfo: Record<AttributeValueId, AVTree>;
  componentName: string;
  schemaName: string;
}

export interface IncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: WeakReference<EntityKind.IncomingConnections>[];
}

// NOTE: these are OUTGOING
export interface ManagementConnections {
  id: ComponentId;
  connections: Connection[];
}

export interface IncomingConnections {
  id: ComponentId;
  connections: Connection[];
}

export interface BifrostIncomingConnections {
  id: ComponentId;
  connections: Connection[];
}

// FIXME(nick,jobelenus): we should split the connection type into two now that management
// connections have their own MV.
export type Connection =
  | {
      kind: "management";
      fromComponentId: ComponentId;
      toComponentId: ComponentId;
    }
  | {
      kind: "prop";
      fromComponentId: ComponentId;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromPropId: PropId;
      fromPropPath: string;
      toComponentId: ComponentId;
      toPropId: PropId;
      toPropPath: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    };

export interface SecretFormDataView {
  name: string;
  kind: string;
  widgetKind: PropertyEditorPropWidgetKind;
}

export interface SecretDefinition {
  label: string;
  formData: Array<SecretFormDataView>;
}

export interface Secret {
  // The [`id`](SecretId) of a [`Secret`].
  id: SecretId;
  // The name of a [`Secret`] as provided by the user.
  name: string;
  // The definition of a [`Secret`].
  label: string;
  // The description of a [`Secret`] as provided by the user.
  description?: null | string;
  // If the secret can be used on this workspace
  isUsable: boolean;
}

export interface DependentValueComponentList {
  id: WorkspacePk;
  componentIds: ComponentId[];
}

export interface DependentValues {
  id: WorkspacePk;
  componentAttributes: Record<ComponentId, AttributePath[]>;
}

export interface DefaultSubscription {
  componentId: ComponentId;
  path: string;
}

export interface DefaultSubscriptions {
  defaultSubscriptions: Map<string, DefaultSubscription>;
  componentsForSubs: DefaultMap<string, Set<ComponentId>>;
  subsForComponents: DefaultMap<ComponentId, Set<string>>;
}

export const emptyDefaultSubs: DefaultSubscriptions = {
  defaultSubscriptions: new Map(),
  componentsForSubs: new DefaultMap(() => new Set()),
  subsForComponents: new DefaultMap(() => new Set()),
};

export interface AuditLog {
  title: string;
  userId?: null | string;
  userEmail?: null | string;
  userName?: null | string;
  kind: string;
  entityName: string;
  entityType: string;
  timestamp: string;
  changeSetId?: null | string;
  changeSetName?: null | string;
  metadata: Record<string, unknown>;
  authenticationMethod: {
    method: string;
    role?: null | string;
    tokenId?: null | string;
  };
}

export interface CachedSchema {
  id: string;
  name: string;
  defaultVariantId: string;
  variantIds: string[];
}

export interface SchemaProp {
  propId: PropId;
  name: string;
  propType: string;
  description?: string;
  children?: SchemaProp[];
  validationFormat?: string;
  defaultValue?: JSON | number | string | boolean;
  hidden?: boolean;
  docLink?: string;
}

export interface CachedSchemaVariant {
  id: SchemaVariantId;
  variantId: SchemaVariantId;
  displayName: string;
  category: string;
  color: string;
  isLocked: boolean;
  description?: string;
  link?: string;
  assetFuncId: FuncId;
  variantFuncIds: FuncId[];
  isDefaultVariant: boolean;
  domainProps?: SchemaProp;
}

export interface CachedDefaultVariant {
  id: SchemaId; // <<-- notice the difference!
  variantId: SchemaVariantId;
  displayName: string;
  category: string;
  color: string;
  isLocked: boolean;
  description?: string;
  link?: string;
  assetFuncId: FuncId;
  variantFuncIds: FuncId[];
  isDefaultVariant: boolean;
  domainProps?: SchemaProp;
}
