export enum FuncKind {
  Action = "Action",
  Attribute = "Attribute",
  Authentication = "Authentication",
  CodeGeneration = "CodeGeneration",
  Intrinsic = "Intrinsic",
  Qualification = "Qualification",
  SchemaVariantDefinition = "SchemaVariantDefinition",
  Unknwon = "Unknown",
}

export enum CustomizableFuncKind {
  Action = "Action",
  Attribute = "Attribute",
  Authentication = "Authentication",
  CodeGeneration = "CodeGeneration",
  Qualification = "Qualification",
}

// TODO(nick,wendy): this is ugly to use in some places. We probably need to think of a better interface. Blame me, not Wendy.
export function customizableFuncKindToFuncKind(
  customizableFuncKind: CustomizableFuncKind,
): FuncKind {
  switch (customizableFuncKind) {
    case CustomizableFuncKind.Action:
      return FuncKind.Action;
    case CustomizableFuncKind.Attribute:
      return FuncKind.Attribute;
    case CustomizableFuncKind.Authentication:
      return FuncKind.Authentication;
    case CustomizableFuncKind.CodeGeneration:
      return FuncKind.CodeGeneration;
    case CustomizableFuncKind.Qualification:
      return FuncKind.Qualification;
    default:
      throw new Error(
        "this should not be possible since CustomizableFuncKind is a subset of FuncKind",
      );
  }
}

export const CUSTOMIZABLE_FUNC_TYPES = {
  [CustomizableFuncKind.Action]: {
    pluralLabel: "Actions",
    singularLabel: "Action",
  },
  [CustomizableFuncKind.Attribute]: {
    pluralLabel: "Attributes",
    singularLabel: "Attribute",
  },
  [CustomizableFuncKind.Authentication]: {
    pluralLabel: "Authentications",
    singularLabel: "Authentication",
  },
  [CustomizableFuncKind.CodeGeneration]: {
    pluralLabel: "Code Generations",
    singularLabel: "Code Generation",
  },
  [CustomizableFuncKind.Qualification]: {
    pluralLabel: "Qualifications",
    singularLabel: "Qualification",
  },
};

export const isCustomizableFuncKind = (f: FuncKind) =>
  f in CUSTOMIZABLE_FUNC_TYPES;

export enum FuncArgumentKind {
  Array = "Array",
  Boolean = "Boolean",
  Integer = "Integer",
  Json = "Json",
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
