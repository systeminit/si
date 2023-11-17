import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";

import storage from "local-storage-fallback";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";

import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import router from "@/router";
import { UserId } from "@/store/auth.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useFeatureFlagsStore } from "./feature_flags.store";

export type ChangeSetId = string;

export function changeSetIdNil(): string {
  return "00000000000000000000000000";
}

export function useChangeSetsStore() {
  const featureFlagsStore = useFeatureFlagsStore();
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;

  return addStoreHooks(
    defineStore(`w${workspacePk || "NONE"}/change-sets`, {
      state: () => ({
        changeSetsById: {} as Record<ChangeSetId, ChangeSet>,
        selectedChangeSetId: null as ChangeSetId | null,
        changeSetsWrittenAtById: {} as Record<ChangeSetId, Date>,
        creatingChangeSet: false as boolean,
        postApplyActor: null as string | null,
        changeSetApprovals: {} as Record<UserId, string>,
      }),
      getters: {
        allChangeSets: (state) => _.values(state.changeSetsById),
        openChangeSets(): ChangeSet[] | null {
          return _.filter(this.allChangeSets, (cs) =>
            [ChangeSetStatus.Open, ChangeSetStatus.NeedsApproval].includes(
              cs.status,
            ),
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
        async APPLY_CHANGE_SET_VOTE(vote: string) {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest({
            method: "post",
            url: "change_set/merge_vote",
            params: {
              vote,
              visibility_change_set_pk: this.selectedChangeSetId,
            },
          });
        },
        async BEGIN_APPROVAL_PROCESS() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest({
            method: "post",
            url: "change_set/begin_approval_process",
            params: {
              visibility_change_set_pk: this.selectedChangeSetId,
            },
          });
        },
        async CANCEL_APPROVAL_PROCESS() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest({
            method: "post",
            url: "change_set/cancel_approval_process",
            params: {
              visibility_change_set_pk: this.selectedChangeSetId,
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
          {
            eventType: "ChangeSetApplied",
            callback: (data) => {
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              const { changeSetPk, userPk } = data as any as {
                changeSetPk: string;
                userPk: UserId;
              };
              const changeSet = this.changeSetsById[changeSetPk];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Applied;
                if (
                  this.selectedChangeSetId === changeSetPk &&
                  featureFlagsStore.MUTLIPLAYER_CHANGESET_APPLY
                ) {
                  this.postApplyActor = userPk;
                }
                this.changeSetsById[changeSetPk] = changeSet;
              }
            },
          },
          {
            eventType: "ChangeSetBeginApprovalProcess",
            callback: (data) => {
              if (this.selectedChangeSetId === data.changeSetPk) {
                this.changeSetApprovals = {};
              }
              const changeSet = this.changeSetsById[data.changeSetPk];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.NeedsApproval;
                changeSet.mergeRequestedAt = new Date().toISOString();
                changeSet.mergeRequestedByUserId = data.userPk;
              }
            },
          },
          {
            eventType: "ChangeSetCancelApprovalProcess",
            callback: (data) => {
              if (this.selectedChangeSetId === data.changeSetPk) {
                this.changeSetApprovals = {};
              }
              const changeSet = this.changeSetsById[data.changeSetPk];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Open;
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
          {
            eventType: "ChangeSetMergeVote",
            callback: (data) => {
              if (this.selectedChangeSetId === data.changeSetPk) {
                this.changeSetApprovals[data.userPk] = data.vote;
              }
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
