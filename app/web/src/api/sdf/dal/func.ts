export enum FuncBackendKind {
  Array = "Array",
  Boolean = "Boolean",
  Identity = "Identity",
  Integer = "Integer",
  JsQualification = "JsQualification",
  JsResourceSync = "JsResourceSync",
  JsCodeGeneration = "JsCodeGeneration",
  JsAttribute = "JsAttribute",
  Map = "Map",
  PropObject = "PropObject",
  String = "String",
  Unset = "Unset",
  Json = "Json",
  ValidateStringValue = "ValidateStringValue",
}

export interface Func {
  id: number;
  handler: string;
  kind: FuncBackendKind;
  name: string;
  description?: string;
  code?: string;
}
