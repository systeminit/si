export enum FuncBackendKind {
  Array = "Array",
  Boolean = "Boolean",
  Identity = "Identity",
  Integer = "Integer",
  JsQualification = "JsQualification",
  JsCommand = "JsCommand",
  JsConfirmation = "JsConfirmation",
  JsCodeGeneration = "JsCodeGeneration",
  JsAttribute = "JsAttribute",
  JsValidation = "JsValidation",
  Map = "Map",
  PropObject = "PropObject",
  String = "String",
  Unset = "Unset",
  Json = "Json",
  ValidateStringValue = "ValidateStringValue",
}

export const CUSTOMIZABLE_FUNC_TYPES = {
  [FuncBackendKind.JsQualification]: {
    pluralLabel: "Qualifications",
    singularLabel: "Qualification",
    enableBuiltIn: true,
  },
  [FuncBackendKind.JsAttribute]: {
    pluralLabel: "Attributes",
    singularLabel: "Attribute",
  },
  [FuncBackendKind.JsCodeGeneration]: {
    pluralLabel: "Code Generators",
    singularLabel: "Code Generation",
  },
  [FuncBackendKind.JsConfirmation]: {
    pluralLabel: "Confirmations",
    singularLabel: "Confirmation",
  },
  [FuncBackendKind.JsCommand]: {
    pluralLabel: "Commands",
    singularLabel: "Command",
  },
  [FuncBackendKind.JsValidation]: {
    pluralLabel: "Validations",
    singularLabel: "Validation",
  },
};

export const isCustomizableFuncKind = (f: FuncBackendKind) =>
  f in CUSTOMIZABLE_FUNC_TYPES;

export interface Func {
  id: number;
  handler: string;
  kind: FuncBackendKind;
  name: string;
  description?: string;
  code?: string;
  isBuiltin: boolean;
}

export enum FuncArgumentKind {
  Array = "Array",
  Boolean = "Boolean",
  Integer = "Integer",
  Object = "Object",
  String = "String",
  Map = "Map",
  Any = "Any",
}

export interface FuncArgument {
  id: number;
  name: string;
  kind: FuncArgumentKind;
  elementKind?: FuncArgumentKind;
}
