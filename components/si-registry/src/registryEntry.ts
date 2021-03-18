import ValidatorJS from "validator";

export enum MenuCategory {
  Application = "application",
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
}

export interface ValidatorBase {
  kind: ValidatorKind;
}

export interface ValidatorInt extends ValidatorBase {
  kind: ValidatorKind.Int;
  options: ValidatorJS.IsIntOptions;
}

export interface ValidatorAlphanumeric extends ValidatorBase {
  kind: ValidatorKind.Alphanumeric;
}

export type Validator = ValidatorInt | ValidatorAlphanumeric;

export interface PropBase {
  type: string;
  name: string;
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
export type ItemProp =
  | Omit<PropString, "name">
  | Omit<PropNumber, "name">
  | Omit<PropObject, "name">
  | Omit<PropArray, "name">;

export interface RegistryEntry {
  entityType: string;
  ui?: RegistryEntryUi;
  properties: Prop[];
}
