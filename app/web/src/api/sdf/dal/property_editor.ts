// Setting a value cold from a schema
// * attribute context
//   * requires a prop id
// * fundamental data type (string/number/bool/map/array)
// * require a key/index (map or array)

export enum PropertyEditorPropKind {
  Array = "array",
  Boolean = "boolean",
  Integer = "integer",
  Object = "object",
  String = "string",
  Map = "map",
}

export enum PropertyEditorPropWidgetKind {
  Array = "array",
  Checkbox = "checkbox",
  Header = "header",
  Map = "map",
  SecretSelect = "secretSelect",
  Text = "text",
}

export interface PropertyEditorProp {
  id: number;
  name: string;
  kind: PropertyEditorPropKind;
  widgetKind: PropertyEditorPropWidgetKind;
  docLink?: string;
}

export interface PropertyEditorSchema {
  rootPropId: number;
  props: { [id: number]: PropertyEditorProp };
  childProps: {
    [key: number]: Array<number>;
  };
}

export interface PropertyEditorValue {
  id: number;
  propId: number;
  key?: string;
  value: unknown;
}

export interface PropertyEditorValues {
  rootValueId: number;
  values: { [id: number]: PropertyEditorValue };
  childValues: {
    [key: number]: Array<number>;
  };
}

export interface PropertyEditorChangeValue {
  valueId: number;
  changed: boolean;
}

export interface PropertyEditorChangeValues {
  changedValues: {
    [valueId: number]: PropertyEditorChangeValue;
  };
}

export interface PropertyEditorValidationError {
  message: string;
  level?: string;
  kind?: string;
  link?: string;
}

export interface PropertyEditorValidation {
  valueId: number;
  valid: boolean;
  errors: Array<PropertyEditorValidationError>;
}

export interface PropertyEditorValidations {
  validations: {
    [valueId: number]: PropertyEditorValidation;
  };
}

export interface UpdatedProperty {
  propId: number;
  valueId: number;
  value: unknown;
}
