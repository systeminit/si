import ValidatorJS from "validator";
import { Optional } from "utility-types";

export enum SchematicKind {
  Deployment = "deployment",
  Component = "component",
}

export enum MenuCategory {
  Application = "application",
  Service = "service",
  Docker = "docker",
  Kubernetes = "kubernetes",
  Helm = "helm",
}

interface RegistryEntryUiHidden {
  menuCategory?: never;
  menuDisplayName?: never;
  schematicKinds?: never;
  hidden: true;
}

interface RegistryEntryUiPresent {
  menuCategory: MenuCategory;
  menuDisplayName: string;
  schematicKinds: SchematicKind[];
  hidden?: never;
}

export type RegistryEntryUi = RegistryEntryUiPresent | RegistryEntryUiHidden;

export enum ValidatorKind {
  Alphanumeric = "alphanumeric",
  Int = "int",
  Regex = "regex",
}

export interface ValidatorBase {
  kind: ValidatorKind;
  link?: string;
  messsage?: string;
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

export type Validator = ValidatorInt | ValidatorAlphanumeric | ValidatorRegex;

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
  | WidgetUnknown;

export interface PropBase {
  type: string;
  name: string;
  widget?: Widgets;
  displayName?: string;
  required?: boolean;
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
}

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
  types: string[] | "implementations";
  edgeKind: "deployment" | "component" | "configures";
  arity: Arity;
  required?: boolean;
}

export enum NodeKind {
  Concept = "concept",
  Implementation = "implementation",
  Concrete = "concrete",
}

export interface RegistryEntry {
  entityType: string;
  nodeKind: NodeKind;
  ui?: RegistryEntryUi;
  implements?: string[];
  inputs: Input[];
  properties: Prop[];
  qualifications?: Qualification[];
  commands?: Command[];
  actions?: Action[];
}
