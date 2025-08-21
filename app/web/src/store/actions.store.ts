import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";
import { Resource } from "@/api/sdf/dal/resource";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { DefaultMap } from "@/utils/defaultmap";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ActionId,
  ActionKind,
  ActionPrototype,
  ActionPrototypeId,
  ActionState,
  ActionResultState,
} from "@/api/sdf/dal/action";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export type DeprecatedActionStatus =
  | "success"
  | "failure"
  | "running"
  | "error"
  | "unstarted";

export type DeprecatedActionRunnerId = string;
export type DeprecatedActionRunner = {
  id: DeprecatedActionRunnerId;
  status: DeprecatedActionStatus;
  actionKind: string;
  schemaName: string;
  componentName: string;
  componentId: ComponentId;
  resource?: Resource | null;
  startedAt?: string;
  finishedAt?: string;
  displayName?: string;
};
export type DeprecatedActionBatchId = string;
export type DeprecatedActionBatch = {
  id: DeprecatedActionBatchId;
  status?: DeprecatedActionStatus;
  author: string;
  actors?: string[];
  actions: DeprecatedActionRunner[];
  startedAt: string | null;
  finishedAt: string | null;
};

export type ComponentAndAction = {
  actionPrototypeId: ActionPrototypeId;
  actionInstanceId?: ActionId;
  componentId?: ComponentId | null;
  actor?: string;
} & Omit<ActionPrototype, "id">;

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

export interface ActionHistoryView extends ActionView {
  funcRunId: FuncRunId;
  result: ActionResultState;
  originatingChangeSetName: string;
  updatedAt: string;
  resourceResult?: string;
  codeExecuted?: string;
  logs?: string;
  arguments?: string;
  componentName: string;
  schemaName: string;
}

export interface ChangeSetDetail {
  changeSetId: ChangeSetId;
  changeSetName: string;
  timestamp?: Date;
}

export const actionKindToAbbreviation = (actionKind: ActionKind) => {
  return {
    Create: "CRT",
    Destroy: "DLT",
    Refresh: "RFH",
    Manual: "MNL",
    Update: "UPT",
  }[actionKind];
};

export const actionIconClass = (kind: ActionKind) => {
  return {
    Create: "text-success-600",
    Destroy: "text-destructive-500 dark:text-destructive-600",
    Refresh: "text-action-600",
    Manual: "text-action-600",
    Update: "text-warning-600",
  }[kind];
};

export const actionIcon = (kind: ActionKind) => {
  return {
    Create: "plus",
    Destroy: "trash",
    Refresh: "refresh",
    Manual: "play",
    Update: "tilde",
  }[kind] as IconNames;
};

export const resultIconClass = (result: ActionResultState) => {
  return {
    Success: "text-success-600",
    Failure: "text-destructive-500 dark:text-destructive-600",
    Unknown: "text-warning-600",
  }[result];
};

export const resultIcon = (result: ActionResultState) => {
  return {
    // outlined icons represent status about the simulation
    // filled icons represent status about resources in the real world
    // so we used filled in icons here
    Success: "check-hex",
    Failure: "x-hex",
    Unknown: "question-hex-outline", // TODO, get a non-outlined icon here
  }[result] as IconNames;
};

// STUFF FOR BOTH ACTIONS V1 AND V2

export type FuncRunId = string;

// END STUFF

export const useActionsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const realtimeStore = useRealtimeStore();

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/actions`,
      {
        state: () => ({
          // used in the right rail when looking at a specific asset
          rawActionsByComponentId: {} as Record<ComponentId, ActionPrototype[]>,
          actions: [] as ActionProposedView[],
          actionHistory: [] as ActionHistoryView[],
        }),
        getters: {
          actionsAreInProgress(): boolean {
            return (this.countActionsByState?.[ActionState.Running] || 0) > 0;
          },
          countActionsByState(): Record<string, number> {
            const counts = new DefaultMap<string, number>(() => 0);
            for (const action of this.proposedActions) {
              counts.set(action.state, counts.get(action.kind) + 1);
            }
            return Object.fromEntries(counts);
          },
          countActionsByKind(): Record<string, number> {
            const counts = new DefaultMap<string, number>(() => 0);
            for (const action of this.proposedActions) {
              counts.set(action.kind, counts.get(action.kind) + 1);
            }
            return Object.fromEntries(counts);
          },
          proposedActions(state): ActionProposedView[] {
            if (changeSetsStore.headSelected)
              return Object.values(state.actions);

            return Object.values(state.actions).filter(
              (av) => av.originatingChangeSetId === changeSetId,
            );
          },
          headActions(state): ActionProposedView[] {
            return Object.values(state.actions).filter(
              (av) => av.originatingChangeSetId !== changeSetId,
            );
          },
          historyActions(): ActionHistoryView[] {
            return this.actionHistory;
          },
          historyActionsById(): Map<ActionId, ActionHistoryView> {
            const m = new Map();
            for (const a of this.historyActions) {
              m.set(a.id, a);
            }
            return m;
          },
          historyActionsByFuncRunId(): Map<FuncRunId, ActionHistoryView> {
            const m = new Map();
            for (const a of this.historyActions) {
              m.set(a.funcRunId, a);
            }
            return m;
          },
          historyActionsGrouped(): Map<
            ChangeSetDetail,
            Array<ActionHistoryView>
          > {
            // display actions, in order, and if the action is from a different change set than the previous action display the header
            const r = new DefaultMap<ChangeSetDetail, Array<ActionHistoryView>>(
              () => [],
            );
            let last: ChangeSetDetail | undefined;
            for (const action of this.actionHistory) {
              // this creates the desired header behavior above
              if (action.originatingChangeSetId !== last?.changeSetId) {
                last = {
                  changeSetId: action.originatingChangeSetId,
                  changeSetName: action.originatingChangeSetName,
                } as ChangeSetDetail;
              }
              // always check for the newest timestamp
              const u = new Date(action.updatedAt);
              if (!last.timestamp || last.timestamp < u) last.timestamp = u;

              const arr = r.get(last);
              arr.push(action);
              r.set(last, arr);
            }
            return r;
          },
          listActionsByComponentId(
            state,
          ): DefaultMap<ComponentId, Array<ActionProposedView>> {
            const d = new DefaultMap<ComponentId, Array<ActionProposedView>>(
              () => [],
            );
            state.actions.forEach((a) => {
              const componentId = a.componentId as ComponentId | null;
              if (componentId) {
                const arr = d.get(componentId);
                arr.push(a);
                d.set(componentId, arr);
              }
            });
            return d;
          },
          actionsById(state): Map<ActionId, ActionView> {
            const m = new Map();
            for (const a of state.actions) {
              m.set(a.id, a);
            }
            return m;
          },
        },
        actions: {
          registerRequestsBegin(requestUlid: string, actionName: string) {
            realtimeStore.inflightRequests.set(requestUlid, actionName);
          },
          registerRequestsEnd(requestUlid: string) {
            realtimeStore.inflightRequests.delete(requestUlid);
          },
        },
        onActivated() {
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, []);

          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
