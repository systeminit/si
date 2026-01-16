import { LabelList } from "@/api/sdf/dal/label_list";

export enum EditFieldObjectKind {
  Component = "Component",
  ComponentProp = "ComponentProp",
  Schema = "Schema",
}

export enum EditFieldDataType {
  Array = "Array",
  Boolean = "Boolean",
  Integer = "Integer",
  Map = "Map",
  None = "None",
  Object = "Object",
  String = "String",
}

export interface CheckboxWidgetDal {
  kind: "Checkbox";
}

export interface TextWidgetDal {
  kind: "Text";
}

export interface SelectWidgetDal {
  kind: "Select";
  options: {
    options: LabelList<unknown>;
    default?: unknown;
  };
  default?: unknown;
}

export interface HeaderWidgetDal {
  kind: "Header";
  options: {
    edit_fields: EditFields;
  };
}

export interface ArrayWidgetDal {
  kind: "Array";
  options: {
    entries: EditFields;
  };
}

export interface MapWidgetDal {
  kind: "Map";
  options: {
    entries: EditFields;
  };
}

export type Widget =
  | CheckboxWidgetDal
  | TextWidgetDal
  | SelectWidgetDal
  | HeaderWidgetDal
  | ArrayWidgetDal
  | MapWidgetDal;

export interface RequiredValidator {
  kind: "Required";
}

export type Validator = RequiredValidator;

export interface VisibilityDiffNone {
  kind: "None";
}

export interface VisibilityDiffHead {
  kind: "Head";
  value: unknown;
}

export interface VisibilityDiffChangeSet {
  kind: "ChangeSet";
  value: unknown;
}

export type VisibilityDiff = VisibilityDiffNone | VisibilityDiffChangeSet | VisibilityDiffHead;

export type EditFieldValues = null | boolean | number | string;

export interface EditFieldBaggage {
  attribute_value_id: string;
  parent_attribute_value_id?: number;
  key?: string;
  prop_id: string;
  prop_doc_link?: string;
}

export interface EditField {
  id: string;
  name: string;
  path: Array<string>;
  object_kind: EditFieldObjectKind;
  object_id: string;
  data_type: EditFieldDataType;
  widget: Widget;
  value?: EditFieldValues;
  visibility_diff: VisibilityDiff;
  validation_errors: ValidationErrors;
  baggage?: EditFieldBaggage;
}

export type EditFields = Array<EditField>;

export type ValidationErrors = Array<ValidationError>;

export interface ValidationError {
  message: string;
  level?: string;
  kind?: string;
  link?: string;
}
