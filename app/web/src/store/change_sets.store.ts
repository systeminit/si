import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";

import storage from "local-storage-fallback";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";

import {
  ActionId,
  ChangeSet,
  ChangeSetStatus,
  ActionPrototype,
  NewAction,
} from "@/api/sdf/dal/change_set";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import router from "@/router";
import { useWorkspacesStore } from "./workspaces.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export type ChangeSetId = string;

export function changeSetIdNil(): string {
  return "00000000000000000000000000";
}

export function useChangeSetsStore() {
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;

  return addStoreHooks(
    defineStore(`w${workspacePk || "NONE"}/change-sets`, {
      state: () => ({
        changeSetsById: {} as Record<ChangeSetId, ChangeSet>,
        selectedChangeSetId: null as ChangeSetId | null,
        changeSetsWrittenAtById: {} as Record<ChangeSetId, Date>,
        creatingChangeSet: false as boolean,
      }),
      getters: {
        allChangeSets: (state) => _.values(state.changeSetsById),
        openChangeSets(): ChangeSet[] | null {
          return _.filter(
            this.allChangeSets,
            (cs) => cs.status === ChangeSetStatus.Open,
          );
        },
        selectedChangeSet: (state) =>
          state.selectedChangeSetId
            ? state.changeSetsById[state.selectedChangeSetId] ?? null
            : null,
        headSelected: (state) => state.selectedChangeSetId === changeSetIdNil(),

        selectedChangeSetWritten: (state) =>
          state.selectedChangeSetId
            ? state.changeSetsWrittenAtById[state.selectedChangeSetId] ?? null
            : null,

        // expose here so other stores can get it without needing to call useWorkspaceStore directly
        selectedWorkspacePk: () => workspacePk,
        statusByComponentId() {
          const componentsStore = useComponentsStore();

          const all: Record<ComponentId, "success" | "failure"> = {};
          for (const component of componentsStore.allComponents) {
            if (component && !component.resource.data) {
              all[component.id] = component.actions.filter(
                (a: ActionPrototype) => a.name === "create",
              ).length
                ? "failure"
                : "success";
            } else if (component && component.deletedInfo) {
              all[component.id] = component.actions.filter(
                (a: ActionPrototype) => a.name === "delete",
              ).length
                ? "failure"
                : "success";
            } else {
              all[component.id] = "success";
            }
          }
          return all;
        },
      },
      actions: {
        async setActiveChangeset(changeSetPk: string) {
          // We need to force refetch changesets since there's a race condition in which redirects
          // will be triggered but the frontend won't have refreshed the list of changesets
          if (!this.changeSetsById[changeSetPk]) {
            await this.FETCH_CHANGE_SETS();
          }

          const route = router.currentRoute.value;
          await router.push({
            name: route.name ?? undefined,
            params: {
              ...route.params,
              changeSetId: changeSetPk,
            },
          });
        },

        async FETCH_CHANGE_SETS() {
          return new ApiRequest<ChangeSet[]>({
            // TODO: probably want to fetch all change sets, not just open (or could have a filter)
            // this endpoint currently returns dropdown-y data, should just return the change set data itself
            url: "change_set/list_open_change_sets",
            onSuccess: (response) => {
              this.changeSetsById = {};

              for (const changeSet of response) {
                this.changeSetsById[changeSet.pk] = {
                  id: changeSet.pk,
                  pk: changeSet.pk,
                  name: changeSet.name,
                  actions: changeSet.actions,
                  status: changeSet.status,
                };
              }
            },
          });
        },
        async REMOVE_ACTION(id: ActionId) {
          return new ApiRequest<null>({
            method: "post",
            url: "change_set/remove_action",
            params: {
              id,
              visibility_change_set_pk: this.selectedChangeSet?.id,
            },
            onSuccess: () => {},
          });
        },
        async ADD_ACTION(action: NewAction) {
          return new ApiRequest<null>({
            method: "post",
            url: "change_set/add_action",
            params: {
              prototypeId: action.prototypeId,
              componentId: action.componentId,
              visibility_change_set_pk: this.selectedChangeSet?.id,
            },
            onSuccess: () => {},
          });
        },
        async CREATE_CHANGE_SET(name: string) {
          return new ApiRequest<{ changeSet: ChangeSet }>({
            method: "post",
            url: "change_set/create_change_set",
            params: {
              changeSetName: name,
            },
            onSuccess: (response) => {
              this.changeSetsById[response.changeSet.pk] = response.changeSet;
            },
          });
        },
        async APPLY_CHANGE_SET() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest<{ changeSet: ChangeSet }>({
            method: "post",
            url: "change_set/apply_change_set",
            params: {
              changeSetPk: this.selectedChangeSet.pk,
            },
            onSuccess: (response) => {
              this.changeSetsById[response.changeSet.pk] = response.changeSet;
              // could switch to head here, or could let the caller decide...
            },
          });
        },
        // TODO: async CANCEL_CHANGE_SET() {},

        // other related endpoints, not necessarily needed at the moment, but available
        // - change_set/get_change_set
        // - change_set/update_selected_change_set (was just fetching the change set info)

        getAutoSelectedChangeSetId() {
          // returning `false` means we cannot auto select
          if (!this.openChangeSets?.length) return false; // no open change sets
          if (this.openChangeSets.length === 1) {
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            return this.openChangeSets[0]!.pk; // only 1 change set - will auto select it
          }
          // TODO: add logic to for auto-selecting when multiple change sets open
          // - select one created by you
          // - track last selected in localstorage and select that one...
          const lastChangeSetId = storage.getItem(
            `SI:LAST_CHANGE_SET/${workspacePk}`,
          );
          if (!lastChangeSetId) return false;
          if (
            this.changeSetsById[lastChangeSetId]?.status ===
            ChangeSetStatus.Open
          ) {
            return lastChangeSetId;
          }
          return false;
        },
        getGeneratedChangesetName() {
          let latestNum = 0;
          _.each(this.allChangeSets, (cs) => {
            const labelNum = parseInt(cs.name.split(" ").pop() || "");
            if (!_.isNaN(labelNum) && labelNum > latestNum) {
              latestNum = labelNum;
            }
          });
          return `Demo ${latestNum + 1}`;
        },
      },
      onActivated() {
        if (!workspacePk) return;
        this.FETCH_CHANGE_SETS();
        const stopWatchSelectedChangeSet = watch(
          () => this.selectedChangeSet,
          () => {
            // store last used change set (per workspace) in localstorage
            if (this.selectedChangeSet && workspacePk) {
              storage.setItem(
                `SI:LAST_CHANGE_SET/${workspacePk}`,
                this.selectedChangeSet.pk,
              );
            }
          },
          { immediate: true },
        );

        const realtimeStore = useRealtimeStore();
        // TODO: if selected change set gets cancelled/applied, need to show error if by other user, and switch to head...
        realtimeStore.subscribe(this.$id, `workspace/${workspacePk}`, [
          {
            eventType: "ChangeSetCreated",
            callback: this.FETCH_CHANGE_SETS,
          },
          {
            eventType: "ChangeSetCancelled",
            callback: this.FETCH_CHANGE_SETS,
          },
          // TODO(Theo/Wendy) - for multiplayer support, we should add code to react if the change set you are using is merged by someone else
          {
            eventType: "ChangeSetApplied",
            callback: (id) => {
              const changeSet = this.changeSetsById[id];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Applied;
                this.changeSetsById[id] = changeSet;
              }
            },
          },
          {
            eventType: "ChangeSetWritten",
            callback: (cs) => {
              // we'll update a timestamp here so individual components can watch this to trigger something if necessary
              // hopefully with more targeted realtime updates we won't need this, but could be useful for now
              this.changeSetsWrittenAtById[cs] = new Date();
              this.FETCH_CHANGE_SETS();

              // could refetch the change sets here, but not useful right now since no interesting metadata exists on the changeset itself
            },
          },
        ]);

        return () => {
          stopWatchSelectedChangeSet();
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
}
