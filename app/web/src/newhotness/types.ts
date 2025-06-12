import { ComputedRef } from "vue";
import { User } from "@/api/sdf/dal/user";
import { ComponentId } from "@/api/sdf/dal/component";

export interface Context {
  workspacePk: ComputedRef<string>;
  changeSetId: ComputedRef<string>;
  user: User | null;
  onHead: ComputedRef<boolean>;
  headChangeSetId: ComputedRef<string>;
  outgoingCounts: ComputedRef<Record<ComponentId, number>>;
  componentNames: ComputedRef<Record<ComponentId, string>>;
}

export function assertIsDefined<T>(value: T | undefined): asserts value is T {
  if (value === undefined) {
    throw new Error("Value is undefined");
  }
}

export interface ExploreContext {
  viewId: ComputedRef<string>;
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
