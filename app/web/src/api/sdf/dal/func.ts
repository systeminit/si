export enum FuncVariant {
  Attribute = "Attribute",
  CodeGeneration = "CodeGeneration",
  Action = "Action",
  Qualification = "Qualification",
  Validation = "Validation",
  Authentication = "Authentication",
}

export const CUSTOMIZABLE_FUNC_TYPES = {
  [FuncVariant.Action]: {
    pluralLabel: "Actions",
    singularLabel: "Action",
  },
  [FuncVariant.Attribute]: {
    pluralLabel: "Attributes",
    singularLabel: "Attribute",
  },
  [FuncVariant.Authentication]: {
    pluralLabel: "Authentications",
    singularLabel: "Authentication",
  },
  [FuncVariant.CodeGeneration]: {
    pluralLabel: "Code Generations",
    singularLabel: "Code Generation",
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
