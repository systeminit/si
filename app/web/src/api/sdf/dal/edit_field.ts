import { LabelList } from "@/api/sdf/dal/label_list";

export enum EditFieldObjectKind {
  Schema = "Schema",
}

export enum EditFieldDataType {
  String = "String",
  Number = "Number",
  Object = "Object",
  Boolean = "Boolean",
  Map = "Map",
  Array = "Array",
  None = "None",
}

export interface CheckboxWidgetDal {
  kind: "Checkbox";
}

export interface TextWidgetDal {
  kind: "Text";
}

export interface SelectWidgetDal {
  kind: "Select";
  options: LabelList<any>;
  default?: any;
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
    entries: EditFields[];
  };
}

export type Widget =
  | CheckboxWidgetDal
  | TextWidgetDal
  | SelectWidgetDal
  | HeaderWidgetDal
  | ArrayWidgetDal;

export interface RequiredValidator {
  kind: "Required";
}

export type Validator = RequiredValidator;

export interface VisibilityDiffNone {
  kind: "None";
}

export interface VisibilityDiffHead {
  kind: "Head";
  value: any;
}

export interface VisibilityDiffChangeSet {
  kind: "ChangeSet";
  value: any;
}

export type VisibilityDiff =
  | VisibilityDiffNone
  | VisibilityDiffChangeSet
  | VisibilityDiffHead;

export interface EditField {
  id: string;
  name: string;
  path: Array<String>;
  object_kind: EditFieldObjectKind;
  object_id: number;
  data_type: EditFieldDataType;
  widget: Widget;
  value: any;
  visibility_diff: VisibilityDiff;
  validators: Array<Validator>;
}

export type EditFields = Array<EditField>;
