// Setting a value cold from a schema
// * attribute context
//   * requires a prop id
// * fundamental data type (string/number/bool/map/array)
// * require a key/index (map or array)

import { LabelList } from "@/api/sdf/dal/label_list";

export enum PropertyEditorPropKind {
  Array = "array",
  Boolean = "boolean",
  Integer = "integer",
  Object = "object",
  String = "string",
  Map = "map",
}

export interface PropertyEditorPropWidgetKindCodeEditor {
  kind: "codeEditor";
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

export interface PropertyEditorPropWidgetKindPassword {
  kind: "password";
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

export interface PropertyEditorPropWidgetKindSecret {
  kind: "secret";
  options: LabelList<string>;
}

export interface PropertyEditorPropWidgetKindColor {
  kind: "color";
}

export type PropertyEditorPropWidgetKind =
  | PropertyEditorPropWidgetKindText
  | PropertyEditorPropWidgetKindTextArea
  | PropertyEditorPropWidgetKindPassword
  | PropertyEditorPropWidgetKindCheckBox
  | PropertyEditorPropWidgetKindMap
  | PropertyEditorPropWidgetKindInteger
  | PropertyEditorPropWidgetKindHeader
  | PropertyEditorPropWidgetKindArray
  | PropertyEditorPropWidgetKindCodeEditor
  | PropertyEditorPropWidgetKindComboBox
  | PropertyEditorPropWidgetKindSelect
  | PropertyEditorPropWidgetKindSecret
  | PropertyEditorPropWidgetKindColor;

export interface PropertyEditorProp {
  id: string;
  name: string;
  kind: PropertyEditorPropKind;
  widgetKind: PropertyEditorPropWidgetKind;
  docLink?: string;
  isHidden: boolean;
  isReadonly: boolean;
  documentation?: string;
  validationFormat?: string;
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
  canBeSetBySocket: boolean;
  isControlledByIntrinsicFunc: boolean;
  overridden: boolean;
  controllingFuncId: string;
  controllingAttributeValueId: string;
  // TODO(Wendy) - we also need the default funcId and funcName for this prop to tell the user the default func that was overriden
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
