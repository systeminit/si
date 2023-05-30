import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";

import storage from "local-storage-fallback";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";

import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { LabelList } from "@/api/sdf/dal/label_list";
import type { Recommendation } from "@/store/fixes.store";
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
        changeSetsById: null as Record<ChangeSetId, ChangeSet> | null,
        selectedChangeSetId: null as ChangeSetId | null,
        changeSetsWrittenAtById: {} as Record<ChangeSetId, Date>,
      }),
      getters: {
        allChangeSets: (state) => _.values(state.changeSetsById),
        openChangeSets(): ChangeSet[] | null {
          if (!this.changeSetsById) return null;

          return _.filter(
            this.allChangeSets,
            (cs) => cs.status === ChangeSetStatus.Open,
          );
        },
        selectedChangeSet: (state) =>
          state.selectedChangeSetId && state.changeSetsById
            ? state.changeSetsById[state.selectedChangeSetId] ?? null
            : null,

        selectedChangeSetWritten: (state) =>
          state.selectedChangeSetId
            ? state.changeSetsWrittenAtById[state.selectedChangeSetId] ?? null
            : null,

        // expose here so other stores can get it without needing to call useWorkspaceStore directly
        selectedWorkspacePk: () => workspacePk,
      },
      actions: {
        async FETCH_CHANGE_SETS() {
          return new ApiRequest<{ list: LabelList<string> }>({
            // TODO: probably want to fetch all change sets, not just open (or could have a filter)
            // this endpoint currently returns dropdown-y data, should just return the change set data itself
            url: "change_set/list_open_change_sets",
            onSuccess: (response) => {
              // this.changeSetsById = _.keyBy(response.changeSets, "id");

              // endpoint returns a dropdown list so we'll temporarily re-format into ChangeSet data
              const changeSetData = _.map(
                response.list,
                (ci) =>
                  ({
                    id: ci.value,
                    pk: ci.value,
                    name: ci.label,
                    // note: null,
                    status: ChangeSetStatus.Open,
                  } as ChangeSet),
              );

              this.changeSetsById = _.keyBy(changeSetData, "id");
            },
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
              this.changeSetsById ||= {};
              this.changeSetsById[response.changeSet.pk] = response.changeSet;
            },
          });
        },
        async APPLY_CHANGE_SET2(recommendations: Array<Recommendation>) {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest<{ changeSet: ChangeSet }>({
            method: "post",
            url: "change_set/apply_change_set2",
            params: {
              changeSetPk: this.selectedChangeSet.pk,
              list: recommendations.map((r) => ({
                attributeValueId: r.confirmationAttributeValueId,
                componentId: r.componentId,
                actionPrototypeId: r.actionPrototypeId,
              })),
            },
            onSuccess: (response) => {
              this.changeSetsById ||= {};
              this.changeSetsById[response.changeSet.pk] = response.changeSet;
              // could switch to head here, or could let the caller decide...
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
              this.changeSetsById ||= {};
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
          if (this.openChangeSets.length === 1)
            return this.openChangeSets[0]?.pk; // only 1 change set - will auto select it
          // TODO: add logic to for auto-selecting when multiple change sets open
          // - select one created by you
          // - track last selected in localstorage and select that one...
          const lastChangeSetId = storage.getItem(
            `SI:LAST_CHANGE_SET/${workspacePk}`,
          );
          if (!lastChangeSetId) return false;
          if (
            this.changeSetsById![lastChangeSetId]?.status ===
            ChangeSetStatus.Open
          ) {
            return lastChangeSetId;
          }
          return false;
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
              const changeSet = (this.changeSetsById ?? {})[id];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Applied;
                this.changeSetsById![id] = changeSet;
              }
            },
          },
          {
            eventType: "ChangeSetWritten",
            callback: (cs) => {
              // we'll update a timestamp here so individual components can watch this to trigger something if necessary
              // hopefully with more targeted realtime updates we won't need this, but could be useful for now
              this.changeSetsWrittenAtById[cs] = new Date();

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
