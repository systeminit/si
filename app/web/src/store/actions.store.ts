import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "./change_sets.store";
import { ComponentId, useComponentsStore } from "./components.store";
import { useFixesStore } from "./fixes.store";

export type ActionStatus = "failure" | "success";

export type ActionPrototypeId = string;
export type ActionInstanceId = string;

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
}

export type FullAction = {
  actionPrototypeId: ActionPrototypeId;
  actionInstanceId?: ActionId;
  componentId?: ComponentId;
} & Omit<ActionPrototype, "id">;

export type ProposedAction = FullAction & {
  actionInstanceId: ActionId;
  componentId: ComponentId;
};

export const useActionsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/actions`,
      {
        state: () => ({}),
        getters: {
          proposedActions(): ProposedAction[] {
            return _.compact(
              _.map(changeSetsStore.selectedChangeSet?.actions, (a) => {
                return _.find(
                  this.actionsByComponentId[a.componentId],
                  (pa) => a.actionPrototypeId === pa.actionPrototypeId,
                );
              }) as ProposedAction[],
            );
          },

          actionsByComponentId(): Record<ComponentId, FullAction[]> {
            const componentsStore = useComponentsStore();
            return _.mapValues(
              componentsStore.componentsById,
              (component, componentId) => {
                return _.compact(
                  _.map(component.actions, (actionPrototype) => {
                    if (actionPrototype.name === "refresh") return;
                    const actionInstance: ActionInstance | undefined = _.find(
                      changeSetsStore.selectedChangeSet?.actions,
                      (pa) =>
                        pa.componentId === componentId &&
                        pa.actionPrototypeId === actionPrototype.id,
                    );

                    return {
                      actionPrototypeId: actionPrototype.id,
                      actionInstanceId: actionInstance?.id,
                      componentId: actionInstance?.componentId,
                      ..._.omit(actionPrototype, "id"),
                    };
                  }),
                );
              },
            );
          },

          // Note(paulo): temporary hack, until we implement a better UI for reconciliation
          requiredActionsByComponentId(): Record<
            ComponentId,
            ActionPrototype[]
          > {
            const componentsStore = useComponentsStore();
            return _.mapValues(
              componentsStore.componentsById,
              (component, _componentId) => {
                const actions = component.actions;

                if (component && !component.resource.data) {
                  return actions.filter((a) => a.name === "create");
                } else if (component?.deletedInfo) {
                  return actions.filter((a) => a.name === "delete");
                } else {
                  return [];
                }
              },
            );
          },

          // TODO: this doesnt really make sense, but was just keeping the confirmations ui in place from actions
          actionStatusByComponentId(): Record<ComponentId, ActionStatus> {
            return _.mapValues(
              this.requiredActionsByComponentId,
              (requiredActions) =>
                requiredActions.length ? "failure" : "success",
            );
          },

          actionHistoryByComponentId() {
            const fixesStore = useFixesStore();
            const allHistory = _.flatMap(
              fixesStore.fixBatches,
              (batch) => batch.fixes,
            );
            return _.groupBy(allHistory, (entry) => entry.componentId);
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
        },
        onActivated() {
          if (!changeSetId) return;
        },
      },
    ),
  )();
};
