export enum FuncVariant {
  Attribute = "Attribute",
  Validation = "Validation",
  Qualification = "Qualification",
  CodeGeneration = "CodeGeneration",
  Confirmation = "Confirmation",
  Command = "Command",
}

export enum FuncBackendResponseType {
  Array = "Array",
  Boolean = "Boolean",
  Identity = "Identity",
  Integer = "Integer",
  Map = "Map",
  Object = "Object",
  Qualification = "Qualification",
  CodeGeneration = "CodeGeneration",
  Confirmation = "Confirmation",
  String = "String",
  Unset = "Unset",
  Json = "Json",
  Validation = "Validation",
  Workflow = "Workflow",
  Command = "Command",
}

export const CUSTOMIZABLE_FUNC_TYPES = {
  [FuncVariant.Attribute]: {
    pluralLabel: "Attributes",
    singularLabel: "Attribute",
    enableBuiltIn: true,
  },
  [FuncVariant.CodeGeneration]: {
    pluralLabel: "Code Generations",
    singularLabel: "Code Generation",
  },
  [FuncVariant.Confirmation]: {
    pluralLabel: "Confirmations",
    singularLabel: "Confirmation",
  },
  [FuncVariant.Command]: {
    pluralLabel: "Commands",
    singularLabel: "Command",
  },
  [FuncVariant.Qualification]: {
    pluralLabel: "Qualifications",
    singularLabel: "Qualification",
  },
  [FuncVariant.Validation]: {
    pluralLabel: "Validations",
    singularLabel: "Validation",
  },
};

export const isCustomizableFuncKind = (f: FuncVariant) =>
  f in CUSTOMIZABLE_FUNC_TYPES;

export interface Func {
  id: string;
  handler: string;
  variant: FuncVariant;
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
  id: string;
  name: string;
  kind: FuncArgumentKind;
  elementKind?: FuncArgumentKind;
}
