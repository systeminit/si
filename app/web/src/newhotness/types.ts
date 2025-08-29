import { ComputedRef, Reactive, Ref } from "vue";
import { User } from "@/api/sdf/dal/user";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ComponentDetails,
  ComponentInList,
  SchemaMembers,
} from "@/workers/types/entity_kind_types";
import { SchemaId } from "@/api/sdf/dal/schema";
import { ChangeSet, ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  ActionId,
  ActionKind,
  ActionPrototypeId,
  ActionState,
} from "@/api/sdf/dal/action";
import { FuncRunId } from "./api_composables/func_run";

// === IDs ===
export type Ulid = string;

export type AttributeValueId = string;
export type ChangeSetApprovalId = string;
export type UserId = string;
export type UserPk = string;
export type WorkspacePk = string;

// === EVERYTHING ELSE ===
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
  userWorkspaceFlags: Ref<Record<string, boolean>>;
  onHead: ComputedRef<boolean>;
  headChangeSetId: Ref<string>;
  outgoingCounts: ComputedRef<Record<ComponentId, number>>;
  componentDetails: ComputedRef<ComponentDetails>;
  schemaMembers: ComputedRef<Record<SchemaId, SchemaMembers>>;
  queriesEnabled: Ref<boolean>;
  reopenOnboarding: () => void;
}

export function assertIsDefined<T>(value: T | undefined): asserts value is T {
  if (value === undefined) {
    throw new Error("Value is undefined");
  }
}

export interface ComponentsHaveActionsWithState {
  failed: Set<ComponentId>;
  running: Set<ComponentId>;
}

export type GridMode =
  | { mode: "default"; label: "" }
  | { mode: "pinned"; componentId: ComponentId; label: "" }
  | { mode: "defaultSubscriptions"; label: "" }
  | { mode: "groupBy"; criteria: "diff"; label: "Diff Status" }
  | {
      mode: "groupBy";
      criteria: "qualification";
      label: "Qualification Status";
    }
  | { mode: "groupBy"; criteria: "upgrade"; label: "Upgradeable" }
  | { mode: "groupBy"; criteria: "schemaName"; label: "Schema Name" }
  | { mode: "groupBy"; criteria: "resource"; label: "Resource" }
  | {
      mode: "groupBy";
      criteria: "incompatibleComponents";
      label: "Incompatible Components";
    };

export interface ExploreContext {
  viewId: ComputedRef<string>;
  upgradeableComponents: ComputedRef<Set<string>>;
  showSkeleton: ComputedRef<boolean>;
  lanesCount: ComputedRef<number>;
  focusedComponentIdx: Ref<number | undefined>;
  selectedComponentsMap: ComputedRef<Record<number, ComponentInList>>;
  focusedComponent: ComputedRef<ComponentInList | undefined>;
  componentsHaveActionsWithState: ComputedRef<ComponentsHaveActionsWithState>;
  selectedComponentIndexes: Reactive<Set<number>>;
  componentsPendingActionNames: ComputedRef<
    Map<ComponentId, Record<string, { count: number; hasFailed: boolean }>>
  >;
  allVisibleComponents: ComputedRef<ComponentInList[]>;
  hasMultipleSections: ComputedRef<boolean>;
  focusedComponentRef: Ref<HTMLElement | undefined>;
  gridMode: Ref<GridMode>;
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

export interface ActionView {
  id: ActionId;
  actor?: string; // TODO i dont see this on the backend
  prototypeId: ActionPrototypeId;
  componentId: ComponentId | null;
  name: string;
  description?: string;
  kind: ActionKind;
  originatingChangeSetId: ChangeSetId;
  funcRunId?: FuncRunId;
}

export interface ActionProposedView extends ActionView {
  state: ActionState;
  myDependencies: ActionId[];
  dependentOn: ActionId[];
  holdStatusInfluencedBy: ActionId[];
  componentSchemaName?: string;
  componentName?: string;
}

export interface WorkspaceUser {
  id: string;
  name: string;
  email: string;
}

export interface ChangeSetApprovalRequirement {
  entityId: Ulid;
  entityKind: string;
  requiredCount: number;
  isSatisfied: boolean;
  applicableApprovalIds: ChangeSetApprovalId[];
  approverGroups: Record<string, string[]>;
  approverIndividuals: string[];
}

export interface ChangeSetApproval {
  id: ChangeSetApprovalId;
  userId: UserId;
  status: "Approved" | "Rejected";
  isValid: boolean; // is this approval "out of date" based on the checksum
}

export interface ApprovalData {
  requirements: ChangeSetApprovalRequirement[];
  latestApprovals: ChangeSetApproval[];
}

// Bulk editing blanks inputs
export interface AttributeInputContext {
  blankInput: boolean;
}
