// Setting a value cold from a schema
// * attribute context
//   * requires a prop id
// * fundamental data type (string/number/bool/map/array)
// * require a key/index (map or array)

import { DoubleLabelList, LabelList } from "@/api/sdf/dal/label_list";
import { ComponentId, AttributePath } from "./component";

export enum PropertyEditorPropKind {
  Array = "array",
  Boolean = "boolean",
  Integer = "integer",
  Object = "object",
  String = "string",
  Map = "map",
  Json = "json",
}

export interface PropertyEditorPropWidgetKindCodeEditor {
  kind: "codeEditor";
}

export interface PropertyEditorPropWidgetKindArray {
  kind: "array";
}

export interface PropertyEditorpropWidgetKindRequirement {
  kind: "requirement";
}

export interface PropertyEditorPropWidgetKindUsers {
  kind: "users";
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

export interface PropertyEditorPropWidgetKindSocketConnection {
  kind: "socketConnection";
  options: DoubleLabelList<string>;
  isSingleArity: boolean;
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
  | PropertyEditorpropWidgetKindRequirement
  | PropertyEditorPropWidgetKindUsers
  | PropertyEditorPropWidgetKindCodeEditor
  | PropertyEditorPropWidgetKindComboBox
  | PropertyEditorPropWidgetKindSelect
  | PropertyEditorPropWidgetKindSecret
  | PropertyEditorPropWidgetKindColor
  | PropertyEditorPropWidgetKindSocketConnection;

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
  defaultCanBeSetBySocket: boolean;
  isOriginSecret: boolean;
  createOnly: boolean;
}

export interface PropertyEditorSchema {
  rootPropId: string;
  props: { [id: string]: PropertyEditorProp };
  childProps: {
    [key: string]: Array<string>;
  };
}

export type ValidationOutputStatus = "Error" | "Failure" | "Success";

export interface ValidationOutput {
  status: ValidationOutputStatus;
  message?: string;
}

export interface PropertyEditorValue {
  id: string;
  propId: string;
  key?: string;
  value: unknown;
  canBeSetBySocket: boolean;
  isFromExternalSource: boolean;
  isControlledByDynamicFunc: boolean;
  isControlledByAncestor: boolean;
  overridden: boolean;
  ancestorManual: boolean;
  validation?: ValidationOutput;
  source?: { component: ComponentId; path: AttributePath };
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
