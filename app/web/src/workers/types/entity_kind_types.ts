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
import { ViewId } from "@/api/sdf/dal/views";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { DefaultMap } from "@/utils/defaultmap";

export enum EntityKind {
  Component = "Component",
  View = "View",
  ViewList = "ViewList",
  ComponentList = "ComponentList",
  ViewComponentList = "ViewComponentList",
  IncomingConnections = "IncomingConnections",
  IncomingConnectionsList = "IncomingConnectionsList",
  SchemaVariantCategories = "SchemaVariantCategories",
  SchemaVariant = "SchemaVariant",
  ActionViewList = "ActionViewList",
  ActionPrototypeViewList = "ActionPrototypeViewList",
  SecretList = "SecretList",
  SecretDefinitionList = "SecretDefinitionList",
  Secret = "Secret",
  SecretDefinition = "SecretDefinition",
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
interface Reference<T extends EntityKind> {
  id: string;
  checksum: string;
  kind: T;
}

interface WeakReference<T extends EntityKind> {
  id: string;
  kind: T;
}

export type OutgoingConnections = DefaultMap<string, BifrostConnection[]>;

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
  views: Reference<EntityKind.View>[];
}

export interface BifrostSchemaVariantCategories {
  id: string; // change set id
  categories: Categories;
}
export interface EddaSchemaVariantCategories {
  id: string; // change set id
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
  running: number;
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
  uninstalled: "uninstalled";
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
  isSecretDefining: boolean;
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
  components: Reference<EntityKind.Component>[];
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
  secretId: string | null;
}

export interface AVTree {
  parent?: AttributeValueId;
  children: AttributeValueId[];
}

export interface AttributeTree {
  attributeValues: Record<AttributeValueId, AttributeValue>;
  props: Record<PropId, Prop>;
  treeInfo: Record<AttributeValueId, AVTree>;
  secrets: Record<string, EddaSecret>; // we dont need the weakref data when its assigned
}

// EntityKind.IncomingConnectionsList
export interface EddaIncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: Reference<EntityKind.IncomingConnections>[];
}

// EntityKind.IncomingConnections
export interface EddaIncomingConnections {
  id: ComponentId;
  connections: EddaConnection[];
}

// EntityKind.IncomingConnectionsList
export interface BifrostIncomingConnectionsList {
  id: ChangeSetId;
  componentConnections: BifrostComponentConnections[];
}

// EntityKind.IncomingConnections
export interface BifrostComponentConnections {
  id: ComponentId;
  component: BifrostComponent;
  incoming: BifrostConnection[];
  // note: outgoing connections cannot be computed right now
}

export type EddaConnection =
  | {
      kind: "prop";
      fromComponentId: WeakReference<EntityKind.Component>;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromPropId: PropId;
      fromPropPath: string;
      toComponentId: WeakReference<EntityKind.Component>;
      toPropId: PropId;
      toPropPath: string;
      toAttributeValueId: AttributeValueId;
      toAttributeValuePath: string;
    }
  | {
      kind: "socket";
      fromComponentId: WeakReference<EntityKind.Component>;
      fromAttributeValueId: AttributeValueId;
      fromAttributeValuePath: string;
      fromSocketId: OutputSocketId;
      fromSocketName: string;
      toComponentId: WeakReference<EntityKind.Component>;
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

export interface SecretFormDataView {
  name: string;
  kind: string;
  widgetKind: PropertyEditorPropWidgetKind;
}

export interface SecretDefinition {
  id: PropId;
  label: string;
  formData: Array<SecretFormDataView>;
}

export interface EddaSecret {
  id: string;
  name: string;
  label: string;
  description?: string;
  isUsable: boolean;
  definitionId: WeakReference<EntityKind.SecretDefinition>;
}

export interface BifrostSecret {
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
  // The prop that contains the secret definition
  definition: SecretDefinition;
}

export interface EddaSecretDefinitionList {
  id: string;
  secretDefinitions: Array<WeakReference<EntityKind.SecretDefinition>>;
}

export interface EddaSecretList {
  id: string;
  secrets: Array<WeakReference<EntityKind.Secret>>;
}

export interface BifrostSecretList {
  id: string;
  secrets: Array<BifrostSecret>;
}

export interface BifrostSecretDefinitionList {
  id: string;
  secretDefinitions: Array<SecretDefinition>;
}
