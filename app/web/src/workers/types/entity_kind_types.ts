import { ActionProposedView } from "@/store/actions.store";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { SchemaId, SchemaVariantId } from "@/api/sdf/dal/schema";
import { ActionKind, ActionPrototypeId } from "@/api/sdf/dal/action";
import { FuncId } from "@/api/sdf/dal/func";
import { AttributeValueId } from "@/store/status.store";
import { PropId, PropKind } from "@/api/sdf/dal/prop";
import {
  PropertyEditorPropWidgetKind,
  ValidationOutput,
} from "@/api/sdf/dal/property_editor";
import { ViewId } from "@/api/sdf/dal/views";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { DefaultMap } from "@/utils/defaultmap";
import { ComponentInfo } from "./dbinterface";

export enum EntityKind {
  ActionPrototypeViewList = "ActionPrototypeViewList",
  ActionViewList = "ActionViewList",
  AttributeTree = "AttributeTree",
  CachedSchemas = "CachedSchemas",
  Component = "Component",
  ComponentDiff = "ComponentDiff",
  ComponentDetails = "ComponentDetails",
  ComponentInList = "ComponentInList",
  ComponentList = "ComponentList",
  ComponentsInOnlyOneView = "ComponentsInOnlyOneView",
  ComponentsInViews = "ComponentsInViews",
  DefaultSubscriptions = "DefaultSubscriptions",
  DependentValueComponentList = "DependentValueComponentList",
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
  SchemaVariantCategories = "SchemaVariantCategories",
  SecretDefinition = "SecretDefinition",
  View = "View",
  ViewComponentList = "ViewComponentList",
  ViewList = "ViewList",
}

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
  value: string | null;
  componentName: string;
  schemaName: string;
  componentId: string;
  kind: string;
  isOriginSecret: boolean;
  suggestAsSourceFor?: PropSuggestion[];
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

export interface BifrostSchemaVariantCategories {
  id: string; // workspace id
  categories: Categories;
}

export interface EddaSchemaVariantCategories {
  id: string; // workspace id
  categories: Array<{
    displayName: string;
    schemaVariants: Array<{
      type: "uninstalled" | "installed";
      id: string;
    }>;
  }>;
  uninstalled: Record<string, UninstalledVariant>;
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
}

export interface EddaComponent {
  id: ComponentId;
  name: string;
  color?: string;
  schemaName: string;
  schemaId: SchemaId;
  schemaVariantId: WeakReference<EntityKind.SchemaVariant>;
  schemaVariantName: string;
  schemaVariantDescription?: string;
  schemaVariantDocLink?: string;
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
    current?: string;
    diff?: string;
  };
  attributeTree: AttributeTree;
  schemaMembers: WeakReference<EntityKind.SchemaMembers>;
  toDelete: boolean;
}

export interface SchemaMembers {
  id: SchemaId;
  defaultVariantId: SchemaVariantId;
  editingVariantId?: SchemaVariantId;
}

export interface UninstalledVariant {
  schemaId: string;
  schemaName: string;
  displayName: string | null;
  category: string;
  color: string;
  link: string | null;
  description: string | null;
  uninstalled: "uninstalled";
  isLocked: boolean;
}

export type CategoryVariant = SchemaVariant | UninstalledVariant;

export type Categories = {
  displayName: string;
  schemaVariants: CategoryVariant[];
}[];

export interface PropTree {
  props: Record<PropId, Prop>;
  treeInfo: Record<PropId, { parent?: PropId; children: PropId[] }>;
}

export interface MgmtFunction {
  id: string;
  funcId: FuncId;
  description?: string;
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

  isSecretDefining: boolean;
  propTree: PropTree;
  canCreateNewComponents: boolean;

  canContribute: boolean;
  mgmtFunctions: MgmtFunction[];
}

export interface BifrostComponent {
  id: ComponentId;
  name: string;
  color?: string;
  schemaName: string;
  schemaId: SchemaId;
  // Needed for "ComponentInList" usage where the "SchemaVariant" is dropped.
  schemaVariantId: WeakReference<EntityKind.SchemaVariant>;
  schemaVariant: SchemaVariant;
  schemaVariantName: string;
  schemaVariantDescription?: string;
  schemaVariantDocLink?: string;
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
    current?: string;
    diff?: string;
  };
  toDelete: boolean;
}

export type ComponentDiffStatus = "Added" | "None" | "Modified" | "Removed";

export interface ComponentDiff {
  id: ComponentId;
  diffStatus: ComponentDiffStatus;
  attributeDiffs: Record<AttributePath, AttributeDiff>;
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
      old?: undefined;
    }
  | {
      // Removed
      new?: undefined;
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
  $source: AttributeSourceLocation & SimplifiedAttributeSource;
  $value?: unknown;
}

/** The place in the tree this came from */
export interface AttributeSourceLocation {
  /** true if the value came from a "default" or attribute function on the schema */
  fromSchema?: true;
  /** If this came from a dynamic function on a *parent* attribute, this is the path to that attribute */
  fromAncestor?: AttributePath;
}

export type SimplifiedAttributeSource =
  | { component: ComponentId; path: AttributePath }
  | { value: unknown }
  | { prototype: string };

// NOTE: when using `getMany` you don't end up with a BifrostComponent (b/c it doesnt have SchemaVariant)
// You end up with a ComponentInList
export interface ComponentInList {
  id: ComponentId;
  name: string;
  color?: string;
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
  childKind: PropKind;
  widgetKind: PropertyEditorPropWidgetKind;
  docLink?: string;
  documentation?: string;
  validationFormation?: string;
  defaultCanBeSetBySocket: boolean;
  isOriginSecret: boolean;
  secretDefinition?: SecretDefinition;
  createOnly: boolean;
  eligibleForConnection: boolean;
  hidden: boolean;
  suggestSources?: PropSuggestion[];
  suggestAsSourceFor?: PropSuggestion[];
}

export interface PropSuggestion {
  schema: string;
  prop: string;
}

export interface AttributeValue {
  id: AttributeValueId;
  key?: string;
  path: AttributePath;
  propId?: PropId;
  value: string | null;
  canBeSetBySocket: boolean;
  externalSources?: ExternalSource[];
  isControlledByAncestor: boolean;
  isControlledByDynamicFunc: boolean;
  overriden: boolean;
  validation?: ValidationOutput;
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
  parent?: AttributeValueId;
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
  id: string;
  // The name of a [`Secret`] as provided by the user.
  name: string;
  // The definition of a [`Secret`].
  label: string;
  // The description of a [`Secret`] as provided by the user.
  description?: string;
  // If the secret can be used on this workspace
  isUsable: boolean;
}

export interface DependentValueComponentList {
  id: string;
  componentIds: ComponentId[];
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
  userId?: string;
  userEmail?: string;
  userName?: string;
  kind: string;
  entityName: string;
  entityType: string;
  timestamp: string;
  changeSetId?: string;
  changeSetName?: string;
  metadata: Record<string, unknown>;
  authenticationMethod: {
    method: string;
    role?: string;
    tokenId?: string;
  };
}
