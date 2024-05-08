import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { trackEvent } from "@/utils/tracking";
import { Resource, ResourceHealth } from "@/api/sdf/dal/resource";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { DefaultMap } from "@/utils/defaultmap";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useChangeSetsStore } from "./change_sets.store";
import { ComponentId } from "./components.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useFeatureFlagsStore } from "./feature_flags.store";

// ACTIONS V1 STUFF - TODO - THIS IS ALL DEPRECATED AND SHOULD BE REMOVED ONCE ACTIONS V1 IS REMOVED
export type DeprecatedActionInstanceId = string;
export type DeprecatedActionStatus =
  | "success"
  | "failure"
  | "running"
  | "error"
  | "unstarted";

export enum DeprecatedActionKind {
  Create = "create",
  Delete = "delete",
  Other = "other",
  Refresh = "refresh",
}
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
export type DeprecatedProposedAction = DeprecatedActionInstance & {
  kind: DeprecatedActionKind;
};
export interface DeprecatedActionPrototype {
  id: ActionPrototypeId;
  name: string;
  displayName: string;
}
export interface DeprecatedNewAction {
  id: never;
  prototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
  displayName: string;
}
export interface DeprecatedActionInstance {
  id: ActionId;
  actionPrototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
  actor?: string;
  parents: ActionId[];
}
export type DeprecatedFullAction = {
  actionPrototypeId: ActionPrototypeId;
  actionInstanceId?: ActionId;
  componentId?: ComponentId;
  actor?: string;
} & Omit<DeprecatedActionPrototype, "id">;

// ACTIONS V2 STUFF - TODO - ONCE ACTIONS V2 WORKS IT SHOULD REPLACE ACTIONS V1
export enum ActionState {
  Dispatched = "dispatched",
  Failed = "failed",
  OnHold = "on_hold",
  Queued = "queued",
  Running = "running",
}
export enum ActionKind {
  Create = "create",
  Destroy = "destroy",
  Refresh = "refresh",
  Manual = "manual",
  Update = "update",
}
export interface ActionView {
  id: ActionId;
  prototypeId: ActionPrototypeId;
  name: string;
  description?: string;
  kind: ActionKind;
  state: ActionState;
  originating_changeset_id: ChangeSetId;
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
          rawActionsByComponentId: {} as Record<
            ComponentId,
            DeprecatedActionPrototype[]
          >,
          rawProposedActionsById: {} as Record<
            ActionId,
            DeprecatedProposedAction
          >,
          actionBatches: [] as Array<DeprecatedActionBatch>,
          runningActionBatch: undefined as DeprecatedActionBatchId | undefined,
          populatingActionRunners: false,
        }),
        getters: {
          actionsAreInProgress: (state) => !!state.runningActionBatch,
          allFinishedActionBatches(): DeprecatedActionBatch[] {
            return this.actionBatches.filter(
              (f) => f.status !== "running" && f.status !== "unstarted",
            );
          },
          actionsOnBatch() {
            return (actionBatchId: DeprecatedActionBatchId) => {
              for (const batch of this.actionBatches) {
                if (batch.id === actionBatchId) {
                  return batch.actions;
                }
              }
              return [];
            };
          },
          completedActionsOnRunningBatch(): DeprecatedActionRunner[] {
            return _.filter(
              this.actionsOnRunningBatch,
              (runner) => !["running", "unstarted"].includes(runner.status),
            );
          },
          actionsOnRunningBatch(): DeprecatedActionRunner[] {
            if (!this.runningActionBatch) return [];
            return this.actionsOnBatch(this.runningActionBatch);
          },
          rawProposedActions: (state) => _.values(state.rawProposedActionsById),
          countActionsByKind(): Record<string, number> {
            const counts = new DefaultMap<string, number>(() => 0);
            for (const action of this.proposedActions) {
              counts.set(action.kind, counts.get(action.kind) + 1);
            }
            return Object.fromEntries(counts);
          },
          proposedActions(): DeprecatedProposedAction[] {
            // TODO: this code was altering the actual store data, so we had to add a cloneDeep
            // probably want to clean up and avoid the while loop if possible too
            const graph = _.cloneDeep(this.rawProposedActionsById);
            const actions = [];
            let count = 0;
            while (_.keys(graph).length) {
              if (count++ > 1000) {
                throw new Error("infinite loop when flattening actions");
              }
              const removeIds = [];

              const sortedEntries = _.entries(graph);
              sortedEntries.sort(([a], [b]) => a.localeCompare(b));

              for (const [id, action] of sortedEntries) {
                if (action.parents.length === 0) {
                  actions.push(action);
                  removeIds.push(id);
                }
              }

              for (const removeId of removeIds) {
                delete graph[removeId];
                for (const childAction of _.values(graph)) {
                  const index = childAction.parents.findIndex(
                    (parentId) => parentId === removeId,
                  );
                  if (index !== -1) {
                    childAction.parents.splice(index);
                  }
                }
              }
            }
            return actions;
          },
          actionsByComponentId(): Record<ComponentId, DeprecatedFullAction[]> {
            return _.mapValues(
              this.rawActionsByComponentId,
              (actions, componentId) => {
                return _.compact(
                  _.map(actions, (actionPrototype) => {
                    if (actionPrototype.name === "refresh") return;

                    const actionInstance: DeprecatedActionInstance | undefined =
                      _.find(
                        this.rawProposedActions,
                        (pa) =>
                          pa.componentId === componentId &&
                          pa.actionPrototypeId === actionPrototype.id,
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

          actionHistoryByComponentId() {
            const allHistory: DeprecatedActionRunner[] = _.flatMap(
              this.actionBatches,
              (batch) => batch.actions,
            );
            return _.groupBy(allHistory, (entry) => entry.componentId);
          },
        },
        actions: {
          async FETCH_DEPRECATED_QUEUED_ACTIONS() {
            if (changeSetId === changeSetsStore.headChangeSetId) {
              return ApiRequest.noop;
            }
            return new ApiRequest<{
              actions: Record<ActionId, DeprecatedProposedAction>;
            }>({
              method: "get",
              url: "change_set/list_queued_actions",
              params: {
                visibility_change_set_pk: changeSetId,
              },
              onSuccess: (response) => {
                this.rawProposedActionsById = response.actions;
              },
            });
          },
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
                visibility_change_set_pk: changeSetId,
              },
            });
          },
          async REMOVE_ACTION(id: ActionId) {
            return new ApiRequest<null>({
              method: "post",
              url: "change_set/remove_action",
              keyRequestStatusBy: id,
              params: {
                id,
                visibility_change_set_pk: changeSetId,
              },
            });
          },
          async FETCH_COMPONENT_ACTIONS(componentId: ComponentId) {
            return new ApiRequest<{ actions: DeprecatedActionPrototype[] }>({
              url: "component/get_actions",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                visibility_change_set_pk: changeSetId,
              },
              onSuccess: (response) => {
                this.rawActionsByComponentId[componentId] = response.actions;
              },
            });
          },
          async LOAD_DEPRECATED_ACTION_BATCHES() {
            return new ApiRequest<Array<DeprecatedActionBatch>>({
              url: "/action/history",
              onSuccess: (response) => {
                this.actionBatches = response;
                this.runningActionBatch = response.find(
                  (batch) =>
                    !["success", "failure", "error"].includes(
                      batch.status ?? "",
                    ),
                )?.id;
              },
            });
          },
          async LOAD_QUEUED_ACTIONS() {
            return new ApiRequest<Array<ActionView>>({
              url: "/action/load_queued",
              headers: { accept: "application/json" },
              params: {
                visibility_change_set_pk: changeSetId,
              },
              onSuccess: (response) => {
                // TODO - UPDATE THIS FOR ACTIONS V2
                console.log(response);
              },
            });
          },
          // TODO - THIS FUNCTION WILL BE RIPPED OUT WHEN ACTIONS V2 IS READY
          async LOAD_V1_OR_V2() {
            if (featureFlagsStore.IS_ACTIONS_V2) {
              this.LOAD_QUEUED_ACTIONS();
            } else {
              this.LOAD_DEPRECATED_ACTION_BATCHES();
              this.FETCH_DEPRECATED_QUEUED_ACTIONS();
            }
          },
        },
        onActivated() {
          if (!changeSetId) return;

          this.LOAD_V1_OR_V2();

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "ChangeSetWritten",
              callback: () => {
                this.LOAD_V1_OR_V2();
              },
            },
            {
              eventType: "ChangeSetApplied",
              callback: (_update) => {
                this.LOAD_DEPRECATED_ACTION_BATCHES();
                // Short term fix for reactivity issue on apply, since the
                // first load won't have the actions since the rebaser isnt done
                setTimeout(() => this.LOAD_DEPRECATED_ACTION_BATCHES(), 500);
              },
            },
            {
              eventType: "DeprecatedActionAdded",
              callback: (action) => {
                this.rawProposedActionsById[action.id] = action;
              },
            },
            {
              eventType: "DeprecatedActionRemoved",
              callback: (actionId) => {
                delete this.rawProposedActionsById[actionId];
              },
            },
            {
              eventType: "DeprecatedActionRunnerReturn",
              callback: async (update) => {
                const status = update.resource
                  ? update.resource.status ?? ResourceHealth.Unknown
                  : ResourceHealth.Error;
                trackEvent("action_runner_return", {
                  action_runner: update.action,
                  action_status: status,
                  action_runner_id: update.id,
                  action_batch_id: update.batchId,
                });

                const batchIndex = this.actionBatches.findIndex(
                  (batch) => batch.id === update.batchId,
                );
                const batch = this.actionBatches[batchIndex];
                if (!batch) {
                  // Short term fix for reactivity issue on apply, since the
                  // first load won't have the actions since the rebaser isnt done
                  setTimeout(() => this.LOAD_DEPRECATED_ACTION_BATCHES(), 500);
                  return;
                }

                const index = batch.actions.findIndex(
                  (runner) => runner.id === update.id,
                );
                const runner = batch.actions[index];
                if (!runner) {
                  // Short term fix for reactivity issue on apply, since the
                  // first load won't have the actions since the rebaser isnt done
                  setTimeout(() => this.LOAD_DEPRECATED_ACTION_BATCHES(), 500);
                  return;
                }

                switch (status) {
                  case ResourceHealth.Ok:
                    runner.status = "success";
                    break;
                  case ResourceHealth.Warning:
                    runner.status = "error";
                    break;
                  case ResourceHealth.Error:
                    runner.status = "error";
                    break;
                  case ResourceHealth.Unknown:
                    runner.status = "unstarted";
                    break;
                  default:
                    runner.status = "unstarted";
                    break;
                }
                runner.resource = update.resource;
              },
            },
            {
              eventType: "DeprecatedActionBatchReturn",
              callback: async (update) => {
                this.runningActionBatch = undefined;
                trackEvent("action_batch_return", {
                  batch_status: update.status,
                  batch_id: update.id,
                });

                const batchIndex = this.actionBatches.findIndex(
                  (batch) => batch.id === update.id,
                );
                const batch = this.actionBatches[batchIndex];
                if (!batch) {
                  // Short term fix for reactivity issue on apply, since the
                  // first load won't have the actions since the rebaser isnt done
                  setTimeout(() => this.LOAD_DEPRECATED_ACTION_BATCHES(), 500);
                  return;
                }
                batch.status = update.status;
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
