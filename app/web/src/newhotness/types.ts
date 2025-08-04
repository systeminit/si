import { ComputedRef, Ref } from "vue";
import { User } from "@/api/sdf/dal/user";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ComponentDetails,
  SchemaMembers,
} from "@/workers/types/entity_kind_types";
import { SchemaId } from "@/api/sdf/dal/schema";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "./api_composables/si_id";

type InstanceEnvType = "LOCAL" | "PRIVATE" | "SI";

export type AuthApiWorkspace = {
  creatorUserId: string;
  displayName: string;
  id: WorkspacePk;
  pk: WorkspacePk; // not actually in the response, but we backfill
  instanceEnvType: InstanceEnvType;
  instanceUrl: string;
  role: "OWNER" | "EDITOR";
  token: string;
  isHidden: boolean;
  approvalsEnabled: boolean;
};

export interface Workspaces {
  workspaces: ComputedRef<Record<WorkspacePk, AuthApiWorkspace> | undefined>;
}

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
