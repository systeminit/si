import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { trackEvent } from "@/utils/tracking";
import { Resource } from "@/api/sdf/dal/resource";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "./change_sets.store";
import { ComponentId } from "./components.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export type ActionStatus =
  | "success"
  | "failure"
  | "running"
  | "error"
  | "unstarted";

export enum ActionKind {
  Create = "create",
  Delete = "delete",
  Other = "other",
  Refresh = "refresh",
}

export type ActionRunnerId = string;
export type ActionRunner = {
  id: ActionRunnerId;
  status: ActionStatus;
  actionKind: string;
  schemaName: string;
  componentName: string;
  componentId: ComponentId;
  resource?: Resource | null;
  startedAt?: string;
  finishedAt?: string;
  displayName?: string;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
export type ActionBatchId = string;
export type ActionBatch = {
  id: ActionBatchId;
  status?: ActionStatus;
  author: string;
  actors?: string[];
  actions: ActionRunner[];
  startedAt: string | null;
  finishedAt: string | null;
};

export type ActionPrototypeId = string;
export type ActionInstanceId = string;

export type ProposedAction = ActionInstance & { kind: ActionKind };

export interface ActionPrototype {
  id: ActionPrototypeId;
  name: string;
  displayName: string;
}

export interface NewAction {
  id: never;
  prototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
  displayName: string;
}

export type ActionId = string;
export interface ActionInstance {
  id: ActionId;
  actionPrototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
  actor?: string;
  parents: ActionId[];
}

export type FullAction = {
  actionPrototypeId: ActionPrototypeId;
  actionInstanceId?: ActionId;
  componentId?: ComponentId;
  actor?: string;
} & Omit<ActionPrototype, "id">;

export const useActionsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/actions`,
      {
        state: () => ({
          rawActionsByComponentId: {} as Record<ComponentId, ActionPrototype[]>,
          rawProposedActionsById: {} as Record<ActionId, ProposedAction>,
          actionBatches: [] as Array<ActionBatch>,
          runningActionBatch: undefined as ActionBatchId | undefined,
          populatingActionRunners: false,
        }),
        getters: {
          actionsAreInProgress: (state) => !!state.runningActionBatch,
          allFinishedActionBatches(): ActionBatch[] {
            return this.actionBatches.filter(
              (f) => f.status !== "running" && f.status !== "unstarted",
            );
          },
          actionsOnBatch() {
            return (actionBatchId: ActionBatchId) => {
              for (const batch of this.actionBatches) {
                if (batch.id === actionBatchId) {
                  return batch.actions;
                }
              }
              return [];
            };
          },
          completedActionsOnRunningBatch(): ActionRunner[] {
            return _.filter(
              this.actionsOnRunningBatch,
              (runner) => !["running", "unstarted"].includes(runner.status),
            );
          },
          actionsOnRunningBatch(): ActionRunner[] {
            if (!this.runningActionBatch) return [];
            return this.actionsOnBatch(this.runningActionBatch);
          },
          rawProposedActions: (state) => _.values(state.rawProposedActionsById),
          proposedActions(): ProposedAction[] {
            // TODO: this code was altering the actual store data, so we had to add a cloneDeep
            // probably want to clean up and avoid the while loop if possible too
            const graph = _.cloneDeep(this.rawProposedActionsById);
            const actions = [];
            let count = 0;
            while (_.keys(graph).length) {
              if (count++ > 1000)
                throw new Error("infinite loop when flattening actions");
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
          actionsByComponentId(): Record<ComponentId, FullAction[]> {
            return _.mapValues(
              this.rawActionsByComponentId,
              (actions, componentId) => {
                return _.compact(
                  _.map(actions, (actionPrototype) => {
                    if (actionPrototype.name === "refresh") return;

                    const actionInstance: ActionInstance | undefined = _.find(
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
            const allHistory: ActionRunner[] = _.flatMap(
              this.actionBatches,
              (batch) => batch.actions,
            );
            return _.groupBy(allHistory, (entry) => entry.componentId);
          },
        },
        actions: {
          async FETCH_QUEUED_ACTIONS() {
            if (changeSetId === nilId()) return ApiRequest.noop;
            return new ApiRequest<{
              actions: Record<ActionId, ProposedAction>;
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
            return new ApiRequest<{ actions: ActionPrototype[] }>({
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
          async LOAD_ACTION_BATCHES() {
            const head = changeSetsStore.allChangeSets.find(
              (cs) => cs.baseChangeSetId === nilId(),
            );
            if (!head) throw new Error("no head");
            return new ApiRequest<Array<ActionBatch>>({
              url: "/action/history",
              params: {
                visibility_change_set_pk: head.id,
              },
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
        },
        onActivated() {
          if (!changeSetId) return;
          this.LOAD_ACTION_BATCHES();
          this.FETCH_QUEUED_ACTIONS();

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(
            this.$id,
            `workspace/${workspaceId}/${changeSetId}`,
            [
              {
                eventType: "ChangeSetWritten",
                callback: () => {
                  this.FETCH_QUEUED_ACTIONS();
                  this.LOAD_ACTION_BATCHES();
                },
              },
              {
                eventType: "ChangeSetApplied",
                callback: (_update) => {
                  this.LOAD_ACTION_BATCHES();
                },
              },
              {
                eventType: "ActionAdded",
                callback: () => {
                  this.FETCH_QUEUED_ACTIONS();
                },
              },
              {
                eventType: "ActionRemoved",
                callback: () => {
                  this.FETCH_QUEUED_ACTIONS();
                },
              },
              {
                eventType: "ActionRunnerReturn",
                callback: (update) => {
                  trackEvent("action_runner_return", {
                    action_runner: update.action,
                    action_status: update.status,
                    action_runner_id: update.id,
                    action_batch_id: update.batchId,
                  });

                  this.LOAD_ACTION_BATCHES();
                },
              },
              {
                eventType: "ActionBatchReturn",
                callback: (update) => {
                  this.runningActionBatch = undefined;
                  trackEvent("action_batch_return", {
                    batch_status: update.status,
                    batch_id: update.id,
                  });

                  this.LOAD_ACTION_BATCHES();
                },
              },
            ],
          );

          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
