import ValidatorJS from "validator";
import { Optional } from "utility-types";

export enum InternalHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
}

export enum SchematicKind {
  Deployment = "deployment",
  Component = "component",
}

export interface RegistryEntryUiHidden {
  menu?: never;
  hidden: true;
}

export interface RegistryEntryUiMenuItem {
  name: string;
  menuCategory: string[];
  schematicKind: SchematicKind;
  rootEntityTypes?: string[];
}

export interface RegistryEntryUiPresent {
  menu: RegistryEntryUiMenuItem[];
  hidden?: never;
}

export type RegistryEntryUi = RegistryEntryUiPresent | RegistryEntryUiHidden;

export enum ValidatorKind {
  Alphanumeric = "alphanumeric",
  Int = "int",
  Regex = "regex",
  IsIn = "isIn",
  Required = "required",
}

export interface ValidatorBase {
  kind: ValidatorKind;
  link?: string;
  messsage?: string;
  userDefined?: boolean;
}

export interface ValidatorRegex extends ValidatorBase {
  kind: ValidatorKind.Regex;
  regex: string;
  message: string;
}

export interface ValidatorInt extends ValidatorBase {
  kind: ValidatorKind.Int;
  options?: ValidatorJS.IsIntOptions;
}

export interface ValidatorAlphanumeric extends ValidatorBase {
  kind: ValidatorKind.Alphanumeric;
}

export interface ValidatorIsIn extends ValidatorBase {
  kind: ValidatorKind.IsIn;
  values: string[];
}

export interface ValidatorRequired extends ValidatorBase {
  kind: ValidatorKind.Required;
}

export type Validator =
  | ValidatorInt
  | ValidatorAlphanumeric
  | ValidatorRegex
  | ValidatorRequired
  | ValidatorIsIn;

export interface WidgetBase {
  name: string;
}

export interface WidgetText extends WidgetBase {
  name: "text";
}

export interface WidgetPassword extends WidgetBase {
  name: "password";
}

export interface WidgetCheckbox extends WidgetBase {
  name: "checkbox";
}

export interface WidgetNumber extends WidgetBase {
  name: "number";
}

export interface WidgetTextArea extends WidgetBase {
  name: "textArea";
}

export interface WidgetSelectOptionsItems {
  items: { value: string | number; label: string }[];
}

export type WidgetSelectOptions = WidgetSelectOptionsItems;

export interface WidgetSelect extends WidgetBase {
  name: "select";
  options: WidgetSelectOptions;
}

export interface WidgetSelectFromInput extends WidgetBase {
  name: "selectFromInput";
  inputName: string;
}

export interface WidgetSelectFromSecret extends WidgetBase {
  name: "selectFromSecret";
  secretKind: string;
}

export interface WidgetUnknown extends WidgetBase {
  name: "unknown";
}

export type Widgets =
  | WidgetText
  | WidgetCheckbox
  | WidgetNumber
  | WidgetPassword
  | WidgetTextArea
  | WidgetSelect
  | WidgetSelectFromInput
  | WidgetSelectFromSecret
  | WidgetUnknown;

export interface EditPartialCategory {
  kind: "category";
  name: string;
  items: EditPartialItem[];
}

export interface EditPartialItem {
  kind: "item";
  name: string;
  propertyPaths: string[][];
}

export type EditPartial = EditPartialItem | EditPartialCategory;

export interface PropBase {
  type: string;
  name: string;
  widget?: Widgets;
  displayName?: string;
  required?: boolean;
  link?: string;
  validation?: Validator[];
}

export interface PropString extends PropBase {
  type: "string";
  defaultValue?: string;
}

export interface PropNumber extends PropBase {
  type: "number";
  defaultValue?: number;
}

export interface PropBool extends PropBase {
  type: "boolean";
  defaultValue?: number;
}

export interface PropObject extends PropBase {
  type: "object";
  properties: Prop[];
  editPartials?: EditPartial[];
}

export interface PropMap extends PropBase {
  type: "map";
  valueProperty: ItemProp;
}

export interface PropArray extends PropBase {
  type: "array";
  itemProperty: ItemProp;
}

export type Prop =
  | PropString
  | PropNumber
  | PropBool
  | PropObject
  | PropArray
  | PropMap;
export type PropScalars = PropString | PropNumber | PropBool;

export type ItemPropString = Optional<PropString, "name">;
export type ItemPropNumber = Optional<PropNumber, "name">;
export type ItemPropBool = Optional<PropBool, "name">;
export type ItemPropObject = Optional<PropObject, "name">;
export type ItemPropMap = Optional<PropMap, "name">;
export type ItemPropArray = Optional<PropArray, "name">;

export type ItemProp =
  | ItemPropString
  | ItemPropNumber
  | ItemPropBool
  | ItemPropObject
  | ItemPropMap
  | ItemPropArray;

export interface Qualification {
  name: string;
  title: string;
  description: string;
  link?: string;
  userDefined?: true;
}

export const allFieldsValidQualification: Qualification = {
  name: "allFieldsValid",
  title: "All fields are valid",
  description: "All the fields must be valid",
};

export interface CommandBase {
  name: string;
  description: string;
}

export interface CommandAlias extends CommandBase {
  name: string;
  description: string;
  aliasTo: string;
  args?: never;
}

export interface CommandDirect extends CommandBase {
  aliastTo?: never;
  args?: Prop[];
}

export type Command = CommandDirect | CommandAlias;

export interface Action {
  name: string;
  args?: Prop[];
}

export enum Arity {
  Many = "many",
  One = "one",
}

export interface Input {
  name: string;
  types: string[] | "implementations" | "dependencies";
  edgeKind: "deployment" | "component" | "configures";
  arity: Arity;
  required?: boolean;
}

export enum NodeKind {
  Concept = "concept",
  Implementation = "implementation",
  Concrete = "concrete",
}

export enum CodeKind {
  YAML = "yaml",
}

export interface Code {
  kind: CodeKind;
}

export interface RegistryEntry {
  entityType: string;
  nodeKind: NodeKind;
  ui?: RegistryEntryUi;
  implements?: string[];
  discoverableFrom?: string[];
  inputs: Input[];
  omitOutputsInSchematic?: SchematicKind[];
  properties: Prop[];
  code?: Code;
  qualifications?: Qualification[];
  commands?: Command[];
  actions?: Action[];
  healthStates?: Record<string, InternalHealth>;
}
