import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { Resource } from "@/api/sdf/dal/resource";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { DefaultMap } from "@/utils/defaultmap";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useFeatureFlagsStore } from "./feature_flags.store";

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

export interface ActionPrototype {
  id: ActionPrototypeId;
  name: string;
  displayName: string;
}

export type ComponentAndAction = {
  actionPrototypeId: ActionPrototypeId;
  actionInstanceId?: ActionId;
  componentId?: ComponentId | null;
  actor?: string;
} & Omit<ActionPrototype, "id">;

export enum ActionState {
  Dispatched = "Dispatched",
  Failed = "Failed",
  OnHold = "OnHold",
  Queued = "Queued",
  Running = "Running",
}

export enum ActionKind {
  Create = "Create",
  Destroy = "Destroy",
  Refresh = "Refresh",
  Manual = "Manual",
  Update = "Update",
}

export interface ActionView {
  id: ActionId;
  actor?: string; // TODO i dont see this on the backend
  prototypeId: ActionPrototypeId;
  componentId: ComponentId | null;
  name: string;
  description?: string;
  kind: ActionKind;
  state: ActionState;
  originatingChangesetId: ChangeSetId;
  myDependencies: ActionId[];
  dependentOn: ActionId[];
  holdStatusInfluencedBy: ActionId[];
}

// STUFF FOR BOTH ACTIONS V1 AND V2

export type ActionPrototypeId = string;
export type ActionId = string;

// END STUFF

export const useActionsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;

  const featureFlagsStore = useFeatureFlagsStore();

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/actions`,
      {
        state: () => ({
          // used in the right rail when looking at a specific asset
          rawActionsByComponentId: {} as Record<ComponentId, ActionPrototype[]>,
          actions: [] as ActionView[],
        }),
        getters: {
          actionBatches() {
            return [] as Array<DeprecatedActionBatch>; // is this history?
          },
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
          proposedActions(state): ActionView[] {
            if (changeSetsStore.headSelected)
              return Object.values(state.actions);

            return Object.values(state.actions).filter(
              (av) => av.originatingChangesetId === changeSetId,
            );
          },
          actionsByComponentId(): Record<ComponentId, ComponentAndAction[]> {
            return _.mapValues(
              this.rawActionsByComponentId,
              (actions, componentId) => {
                return _.compact(
                  _.map(actions, (actionPrototype) => {
                    if (actionPrototype.name === "refresh") return;

                    const actionInstance: ActionView | undefined = _.find(
                      this.actions,
                      (pa: ActionView) => {
                        if (!pa) return false;
                        return (
                          pa.componentId === componentId &&
                          pa.prototypeId === actionPrototype.id
                        );
                      },
                    );

                    return {
                      actionPrototypeId: actionPrototype.id,
                      actionInstanceId: actionInstance?.id,
                      componentId: actionInstance?.componentId,
                      actor: actionInstance?.actor,
                      ..._.omit(actionPrototype, "id"),
                    };
                  }),
                );
              },
            );
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
          async ADD_ACTION(
            componentId: ComponentId,
            actionPrototypeId: ActionPrototypeId,
          ) {
            return new ApiRequest({
              method: "post",
              url: "change_set/add_action",
              keyRequestStatusBy: [componentId, actionPrototypeId],
              params: {
                prototypeId: actionPrototypeId,
                componentId,
                v2: featureFlagsStore.IS_ACTIONS_V2,
                visibility_change_set_pk: changeSetId,
              },
            });
          },
          async FETCH_COMPONENT_ACTIONS(componentId: ComponentId) {
            return new ApiRequest<{ actions: ActionPrototype[] }>({
              url: "component/get_actions",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                v2: featureFlagsStore.IS_ACTIONS_V2,
                visibility_change_set_pk: changeSetId,
              },
              onSuccess: (response) => {
                this.rawActionsByComponentId[componentId] = response.actions;
              },
            });
          },
          async LOAD_ACTIONS() {
            return new ApiRequest<Array<ActionView>>({
              url: "/action/list",
              headers: { accept: "application/json" },
              params: {
                visibility_change_set_pk: changeSetId,
              },
              onSuccess: (response) => {
                this.actions = response;
              },
            });
          },
          async PUT_ACTION_ON_HOLD(ids: ActionId[]) {
            return new ApiRequest<null>({
              method: "post",
              url: "action/put_on_hold",
              keyRequestStatusBy: ids,
              params: {
                ids,
                visibility_change_set_pk: changeSetId,
              },
              optimistic: () => {
                for (const a of this.actions) {
                  if (ids.includes(a.id)) {
                    a.state = ActionState.OnHold;
                  }
                }
              },
            });
          },
          async CANCEL(ids: ActionId[]) {
            return new ApiRequest<null>({
              method: "post",
              url: "action/cancel",
              keyRequestStatusBy: ids,
              params: {
                ids,
                visibility_change_set_pk: changeSetId,
              },
              optimistic: () => {
                for (const idx in this.actions) {
                  const a = this.actions[idx];
                  if (a && ids.includes(a.id)) {
                    delete this.actions[idx];
                  }
                }
              },
            });
          },
          async RETRY(ids: ActionId[]) {
            return new ApiRequest<null>({
              method: "post",
              url: "action/retry",
              keyRequestStatusBy: ids,
              params: {
                ids,
                visibility_change_set_pk: changeSetId,
              },
              optimistic: () => {
                for (const a of this.actions) {
                  if (ids.includes(a.id)) {
                    // its either moving from failed or hold to queued
                    // if its failed it will go queued and start running
                    a.state = ActionState.Queued;
                  }
                }
              },
            });
          },
        },
        onActivated() {
          if (!changeSetId) return;

          this.LOAD_ACTIONS();

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "ActionsListUpdated",
              callback: () => {
                this.LOAD_ACTIONS();
              },
            },
            {
              eventType: "ChangeSetWritten",
              callback: () => {
                this.LOAD_ACTIONS();
              },
            },
            {
              eventType: "ChangeSetApplied",
              callback: (_update) => {
                this.LOAD_ACTIONS();
                // Short term fix for reactivity issue on apply, since the
                // first load won't have the actions since the rebaser isnt done
                // Updated: We may not need this TIMEOUT if new WsEvents fix it!
                setTimeout(() => this.LOAD_ACTIONS(), 500);
              },
            },
          ]);

          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
