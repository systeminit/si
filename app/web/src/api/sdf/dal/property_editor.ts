// Setting a value cold from a schema
// * attribute context
//   * requires a prop id
// * fundamental data type (string/number/bool/map/array)
// * require a key/index (map or array)

import { LabelList } from "@/api/sdf/dal/label_list";

import { FuncBackendKind } from "@/api/sdf/dal/func";

export enum PropertyEditorPropKind {
  Array = "array",
  Boolean = "boolean",
  Integer = "integer",
  Object = "object",
  String = "string",
  Map = "map",
}

export interface PropertyEditorPropWidgetKindArray {
  kind: "array";
}

export interface PropertyEditorPropWidgetKindCheckBox {
  kind: "checkbox";
}

export interface PropertyEditorPropWidgetKindHeader {
  kind: "header";
}

export interface PropertyEditorPropWidgetKindMap {
  kind: "map";
}

export interface PropertyEditorPropWidgetKindText {
  kind: "text";
}

export interface PropertyEditorPropWidgetKindTextArea {
  kind: "textArea";
}

export interface PropertyEditorPropWidgetKindInteger {
  kind: "integer";
}

export interface PropertyEditorPropWidgetKindComboBox {
  kind: "comboBox";
  options?: LabelList<string | number>;
}

export interface PropertyEditorPropWidgetKindSelect {
  kind: "select";
  options?: LabelList<string | number>;
}

export interface PropertyEditorPropWidgetKindSecretSelect {
  kind: "secretSelect";
  options: LabelList<string | number>;
}

export type PropertyEditorPropWidgetKind =
  | PropertyEditorPropWidgetKindText
  | PropertyEditorPropWidgetKindTextArea
  | PropertyEditorPropWidgetKindCheckBox
  | PropertyEditorPropWidgetKindMap
  | PropertyEditorPropWidgetKindInteger
  | PropertyEditorPropWidgetKindHeader
  | PropertyEditorPropWidgetKindArray
  | PropertyEditorPropWidgetKindComboBox
  | PropertyEditorPropWidgetKindSelect
  | PropertyEditorPropWidgetKindSecretSelect;

export interface PropertyEditorProp {
  id: string;
  name: string;
  kind: PropertyEditorPropKind;
  widgetKind: PropertyEditorPropWidgetKind;
  docLink?: string;
  isHidden: boolean;
  isReadonly: boolean;
}

export interface PropertyEditorSchema {
  rootPropId: string;
  props: { [id: string]: PropertyEditorProp };
  childProps: {
    [key: string]: Array<string>;
  };
}

export interface PropertyEditorValue {
  id: string;
  propId: string;
  key?: string;
  value: unknown;
  isFromExternalSource: boolean;
  func: FuncWithPrototypeContext;
}

export interface PropertyEditorValues {
  rootValueId: string;
  values: { [id: string]: PropertyEditorValue };
  childValues: {
    [key: string]: Array<string>;
  };
}

export interface PropertyEditorChangeValue {
  valueId: string;
  changed: boolean;
}

export interface PropertyEditorChangeValues {
  changedValues: {
    [valueId: string]: PropertyEditorChangeValue;
  };
}

export interface PropertyEditorValidationError {
  message: string;
  level?: string;
  kind?: string;
  link?: string;
}

export interface PropertyEditorValidation {
  valueId: string;
  valid: boolean;
  errors: Array<PropertyEditorValidationError>;
}

export interface PropertyEditorValidations {
  validations: PropertyEditorValidation[];
}

export interface UpdatedProperty {
  propId: string;
  valueId: string;
  value: unknown;
  parentValueId?: string;
  key?: string;
}

export interface AddToArray {
  propId: string;
  valueId: string;
  key?: string;
}

export interface AddToMap {
  propId: string;
  valueId: string;
  parentValueId?: string;
  key?: string;
}

export interface PropertyPath {
  displayPath: string[];
  triggerPath: string[];
}

export interface FuncWithPrototypeContext {
  id: string;
  name: string;
  displayName?: string;
  backendKind: FuncBackendKind;
  isBuiltin: boolean;
  attributePrototypeId: string;
  attributeContextComponentId: string;
}
