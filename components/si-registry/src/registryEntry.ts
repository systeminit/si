import ValidatorJS from "validator";
import { Optional } from "utility-types";

export enum MenuCategory {
  Application = "application",
  Docker = "docker",
  Kubernetes = "kubernetes",
  Helm = "helm",
}

interface RegistryEntryUiHidden {
  menuCategory?: never;
  menuDisplayName?: never;
  hidden: true;
}

interface RegistryEntryUiPresent {
  menuCategory: MenuCategory;
  menuDisplayName: string;
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
  options: ValidatorJS.IsIntOptions;
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

export type Widgets = WidgetText;

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

export interface PropObject extends PropBase {
  type: "object";
  properties: Prop[];
}

export interface PropArray extends PropBase {
  type: "array";
  itemProperty: ItemProp;
}

export type Prop = PropString | PropNumber | PropObject | PropArray;
export type PropScalars = PropString | PropNumber;

export type ItemPropString = Optional<PropString, "name">;
export type ItemPropNumber = Optional<PropNumber, "name">;
export type ItemPropObject = Optional<PropObject, "name">;
export type ItemPropArray = Optional<PropArray, "name">;

export type ItemProp =
  | ItemPropString
  | ItemPropNumber
  | ItemPropObject
  | ItemPropArray;

export interface Qualification {
  name: string;
  title: string;
  description: string;
  link?: string;
}

export interface RegistryEntry {
  entityType: string;
  ui?: RegistryEntryUi;
  properties: Prop[];
  qualifications?: Qualification[];
}
