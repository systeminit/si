import { Ref } from "vue";

export interface WSCS {
  workspacePk: Ref<string, string>;
  changeSetId: Ref<string, string>;
}

export function assertIsDefined<T>(value: T | undefined): asserts value is T {
  if (value === undefined) {
    throw new Error("Value is undefined");
  }
}

// Define an enum for function kinds
export enum FunctionKind {
  Action = "action",
  Attribute = "attribute",
  Authentication = "authentication",
  CodeGeneration = "codeGeneration",
  Intrinsic = "intrinsic",
  Management = "management",
}
