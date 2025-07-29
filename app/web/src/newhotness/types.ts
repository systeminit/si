import { ComputedRef, Ref } from "vue";
import { User } from "@/api/sdf/dal/user";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ComponentDetails,
  SchemaMembers,
} from "@/workers/types/entity_kind_types";
import { SchemaId } from "@/api/sdf/dal/schema";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { WorkspaceUser } from "@/store/auth.store";

export interface Context {
  workspacePk: ComputedRef<string>;
  changeSetId: ComputedRef<string>;
  changeSet: Ref<ChangeSet | undefined>;
  approvers: Ref<string[]>;
  user: User | null;
  onHead: ComputedRef<boolean>;
  headChangeSetId: Ref<string>;
  outgoingCounts: ComputedRef<Record<ComponentId, number>>;
  componentDetails: ComputedRef<ComponentDetails>;
  schemaMembers: ComputedRef<Record<SchemaId, SchemaMembers>>;
  queriesEnabled: Ref<boolean>;
  approvalsEnabled: ComputedRef<boolean>;
  workspaceHasOneUser: ComputedRef<boolean>;
  workspaceUsers: ComputedRef<Record<string, WorkspaceUser>>;
}

export function assertIsDefined<T>(value: T | undefined): asserts value is T {
  if (value === undefined) {
    throw new Error("Value is undefined");
  }
}

export interface ExploreContext {
  viewId: ComputedRef<string>;
  upgradeableComponents: ComputedRef<Set<string>>;
  showSkeleton: ComputedRef<boolean>;
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
