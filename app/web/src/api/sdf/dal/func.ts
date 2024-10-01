import { ActionKind, ActionPrototypeId } from "@/api/sdf/dal/action";
import { InputSocketId, OutputSocketId, SchemaVariantId } from "./schema";
import { ComponentId } from "./component";
import { PropId } from "./prop";

export type FuncArgumentId = string;
export type FuncId = string;
export type AttributePrototypeArgumentId = string;
export type AttributePrototypeId = string;

export enum FuncKind {
  Action = "Action",
  Attribute = "Attribute",
  Authentication = "Authentication",
  CodeGeneration = "CodeGeneration",
  Intrinsic = "Intrinsic",
  Qualification = "Qualification",
  SchemaVariantDefinition = "SchemaVariantDefinition",
  Unknown = "Unknown",
}

export enum CustomizableFuncKind {
  Action = "Action",
  Attribute = "Attribute",
  Authentication = "Authentication",
  CodeGeneration = "CodeGeneration",
  Qualification = "Qualification",
}

// TODO(nick,wendy): this is ugly to use in some places. We probably need to think of a better interface. Blame me, not Wendy.
export function customizableFuncKindToFuncKind(
  customizableFuncKind: CustomizableFuncKind,
): FuncKind {
  switch (customizableFuncKind) {
    case CustomizableFuncKind.Action:
      return FuncKind.Action;
    case CustomizableFuncKind.Attribute:
      return FuncKind.Attribute;
    case CustomizableFuncKind.Authentication:
      return FuncKind.Authentication;
    case CustomizableFuncKind.CodeGeneration:
      return FuncKind.CodeGeneration;
    case CustomizableFuncKind.Qualification:
      return FuncKind.Qualification;
    default:
      throw new Error(
        "this should not be possible since CustomizableFuncKind is a subset of FuncKind",
      );
  }
}

export const CUSTOMIZABLE_FUNC_TYPES = {
  [CustomizableFuncKind.Action]: {
    pluralLabel: "Actions",
    singularLabel: "Action",
  },
  [CustomizableFuncKind.Attribute]: {
    pluralLabel: "Attributes",
    singularLabel: "Attribute",
  },
  [CustomizableFuncKind.Authentication]: {
    pluralLabel: "Authentications",
    singularLabel: "Authentication",
  },
  [CustomizableFuncKind.CodeGeneration]: {
    pluralLabel: "Code Generations",
    singularLabel: "Code Generation",
  },
  [CustomizableFuncKind.Qualification]: {
    pluralLabel: "Qualifications",
    singularLabel: "Qualification",
  },
};

export const isCustomizableFuncKind = (f: FuncKind) =>
  f in CUSTOMIZABLE_FUNC_TYPES;

export enum FuncArgumentKind {
  Array = "array",
  Boolean = "boolean",
  Integer = "integer",
  Json = "json",
  Object = "object",
  String = "string",
  Map = "map",
  Any = "any",
}

export interface FuncArgument {
  id: string;
  name: string;
  kind: FuncArgumentKind;
  elementKind?: FuncArgumentKind;
  created_at: IsoDateString;
  updated_at: IsoDateString;
}

export enum FuncBackendKind {
  Array = "Array",
  Boolean = "Boolean",
  Diff = "Diff",
  Identity = "Identity",
  Integer = "Integer",
  JsAction = "JsAction",
  JsAttribute = "JsAction",
  JsAuthentication = "JsAuthentication",
  Json = "Json",
  JsReconciliation = "JsReconciliation",
  JsSchemaVariantDefinition = "JsSchemaVariantDefinition",
  Map = "Map",
  Object = "Object",
  String = "String",
  Unset = "Unset",
  Validation = "Validation",
}
export interface FuncSummary {
  funcId: FuncId;
  kind: FuncKind;
  name: string;
  displayName: string | null;
  description: string | null;
  isLocked: boolean;
  arguments: FuncArgument[];
  backendKind: FuncBackendKind;
  bindings: FuncBinding[];
  types?: string | null;
}

export interface FuncCode {
  funcId: FuncId;
  code: string;
}

export interface AttributeArgumentBinding {
  funcArgumentId: FuncArgumentId;
  attributePrototypeArgumentId: AttributePrototypeArgumentId | null;
  propId: PropId | null;
  inputSocketId: InputSocketId | null;
}

export enum FuncBindingKind {
  Action = "action",
  Attribute = "attribute",
  Authentication = "authentication",
  CodeGeneration = "codeGeneration",
  Qualification = "qualification",
}

export interface Action {
  bindingKind: FuncBindingKind.Action;
  // uneditable
  funcId: FuncId | null;
  schemaVariantId: SchemaVariantId | null;
  actionPrototypeId: ActionPrototypeId | null;
  // editable
  kind: ActionKind | null;
}

export interface Attribute {
  bindingKind: FuncBindingKind.Attribute;
  // uneditable
  funcId: FuncId | null;
  attributePrototypeId: AttributePrototypeId | null;
  // needed on create
  schemaVariantId: SchemaVariantId | null;
  componentId: ComponentId | null;
  // editable
  propId: PropId | null;
  outputSocketId: OutputSocketId | null;
  argumentBindings: AttributeArgumentBinding[];
}

export interface Authentication {
  bindingKind: FuncBindingKind.Authentication;
  funcId: FuncId;
  schemaVariantId: SchemaVariantId;
}

export interface CodeGeneration {
  bindingKind: FuncBindingKind.CodeGeneration;
  funcId: FuncId | null;
  schemaVariantId: SchemaVariantId | null;
  attributePrototypeId: AttributePrototypeId | null;
  componentId: ComponentId | null;
  // editable
  inputs: LeafInputLocation[];
}

export interface Qualification {
  bindingKind: FuncBindingKind.Qualification;
  funcId: FuncId | null;
  schemaVariantId: SchemaVariantId | null;
  attributePrototypeId: AttributePrototypeId | null;
  componentId: ComponentId | null;
  // editable
  inputs: LeafInputLocation[];
}

export type FuncBinding =
  | Action
  | Attribute
  | Authentication
  | CodeGeneration
  | Qualification;

export type LeafInputLocation =
  | "code"
  | "deletedAt"
  | "domain"
  | "resource"
  | "secrets";

export interface BindingWithBackendKind extends Attribute {
  backendKind: FuncBackendKind;
  attributePrototypeId: NonNullable<AttributePrototypeId>;
}

export interface PropDisplay {
  id: PropId;
  path: string;
  name: string;
  value?: PropId | InputSocketId;
  attributePrototypeId?: AttributePrototypeId;
  funcId: FuncId;
}

export interface BindingWithBackendKindAndPropId
  extends BindingWithBackendKind {
  propId: NonNullable<PropId>;
}

export interface IntrinsicDisplay {
  attributePrototypeId: AttributePrototypeId;
  outputSocketId: OutputSocketId;
  socketName: string;
  backendKind: FuncBackendKind;
  value: InputSocketId | PropId | undefined;
  funcId: FuncId;
}

// PSA: this is how to type guard filter so later operations know the field
// is no longer nullable b/c the filter removed any objects where the property was null
export interface BindingWithBackendKindAndOutputSocket
  extends BindingWithBackendKind {
  outputSocketId: NonNullable<OutputSocketId>;
}
