import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { useToast } from "vue-toastification";
import { ulid } from "ulid";
import { URLPattern } from "@si/vue-lib";
import {
  ChangeSet,
  ChangeSetId,
  ChangeSetStatus,
} from "@/api/sdf/dal/change_set";
import { WorkspaceMetadata } from "@/api/sdf/dal/workspace";
import router from "@/router";
import { UserId, useAuthStore } from "@/store/auth.store";
import IncomingChangesMerging from "@/components/toasts/IncomingChangesMerging.vue";
import MovedToHead from "@/components/toasts/MovedToHead.vue";
import RebaseOnBase from "@/components/toasts/RebaseOnBase.vue";
import ChangeSetStatusChanged from "@/components/toasts/ChangeSetStatusChanged.vue";
import { useWorkspacesStore } from "./workspaces.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useRouterStore } from "./router.store";
import handleStoreError from "./errors";
import { useStatusStore } from "./status.store";
import * as heimdall from "./realtime/heimdall";
import { useFeatureFlagsStore } from "./feature_flags.store";

const toast = useToast();

export const diagramUlid = ulid();

export interface StatusWithBase {
  baseHasUpdates: boolean;
  changeSetHasUpdates: boolean;
  conflictsWithBase: boolean;
}

export interface OpenChangeSetsView {
  headChangeSetId: ChangeSetId;
  changeSets: ChangeSet[];
}

export type ChangeSetApprovalId = string;
export type Ulid = string;
export interface ChangeSetApprovalRequirement {
  entityId: Ulid;
  entityKind: string;
  requiredCount: number;
  isSatisfied: boolean;
  applicableApprovalIds: ChangeSetApprovalId[];
  approverGroups: Record<string, string[]>;
  approverIndividuals: string[];
}

export type ApprovalStatus = "Approved" | "Rejected";
export interface ChangeSetApproval {
  id: ChangeSetApprovalId;
  userId: UserId;
  status: ApprovalStatus;
  isValid: boolean; // is this approval "out of date" based on the checksum
}

export interface ApprovalData {
  requirements: ChangeSetApprovalRequirement[];
  latestApprovals: ChangeSetApproval[];
}

export const approverForChangeSet = (
  userId: UserId,
  approvalData: ApprovalData,
) =>
  approvalData.requirements.some((r) =>
    Object.values(r.approverGroups)
      .flat()
      .concat(r.approverIndividuals)
      .includes(userId),
  );

export function useChangeSetsStore() {
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;

  const authStore = useAuthStore();
  const realtimeStore = useRealtimeStore();
  const featureFlagsStore = useFeatureFlagsStore();

  const BASE_API = [
    "v2",
    "workspaces",
    { workspacePk },
    "change-sets",
  ] as URLPattern;

  return addStoreHooks(
    workspacePk,
    undefined,
    defineStore(`w${workspacePk || "NONE"}/change-sets`, {
      state: () => ({
        headChangeSetId: null as ChangeSetId | null,
        changeSetsById: {} as Record<ChangeSetId, ChangeSet>,
        changeSetsWrittenAtById: {} as Record<ChangeSetId, Date>,
        creatingChangeSet: false as boolean,
        postApplyActor: null as string | null,
        postAbandonActor: null as string | null,
        changeSetApprovals: {} as Record<UserId, string>,
        statusWithBase: {} as Record<ChangeSetId, StatusWithBase>,
        defaultApprovers: [] as UserId[],
        changeSetsApprovalData: {} as Record<ChangeSetId, ApprovalData>,
      }),
      getters: {
        currentUserIsDefaultApprover(): boolean {
          const userPk = authStore.user?.pk;
          if (!userPk) return false;
          return this.defaultApprovers.includes(userPk);
        },
        allChangeSets: (state) => _.values(state.changeSetsById),
        changeSetsNeedingApproval(): ChangeSet[] {
          return _.filter(this.allChangeSets, (cs) =>
            [ChangeSetStatus.NeedsApproval].includes(cs.status),
          );
        },
        openChangeSets(): ChangeSet[] {
          return _.filter(this.allChangeSets, (cs) =>
            [
              ChangeSetStatus.Open,
              ChangeSetStatus.NeedsApproval,
              ChangeSetStatus.NeedsAbandonApproval,
              ChangeSetStatus.Rejected,
              ChangeSetStatus.Approved,
            ].includes(cs.status),
          );
        },
        urlSelectedChangeSetId(): ChangeSetId | undefined {
          const route = useRouterStore().currentRoute;
          const id = route?.params?.changeSetId as ChangeSetId | undefined;
          if (id === "head" && this.headChangeSetId) {
            return this.headChangeSetId;
          }
          return id;
        },
        selectedChangeSet(): ChangeSet | null {
          return this.changeSetsById[this.urlSelectedChangeSetId || ""] || null;
        },
        headSelected(): boolean {
          if (this.headChangeSetId) {
            return this.urlSelectedChangeSetId === this.headChangeSetId;
          }
          return false;
        },

        selectedChangeSetLastWrittenAt(): Date | null {
          return (
            this.changeSetsWrittenAtById[this.selectedChangeSet?.id || ""] ??
            null
          );
        },

        selectedChangeSetId(): ChangeSetId | undefined {
          return this.selectedChangeSet?.id;
        },

        // expose here so other stores can get it without needing to call useWorkspaceStore directly
        selectedWorkspacePk: () => workspacePk,
      },
      actions: {
        async setActiveChangeset(changeSetId: string, stayOnView = false) {
          // We need to force refetch changesets since there's a race condition in which redirects
          // will be triggered but the frontend won't have refreshed the list of changesets
          if (!this.changeSetsById[changeSetId]) {
            await this.FETCH_CHANGE_SETS();
          }

          const route = router.currentRoute.value;

          const params = { ...route.params };
          let name = route.name;

          // if abandoning changeset and you were looking at view, it may not exist in HEAD
          if (!stayOnView && name === "workspace-compose-view") {
            name = "workspace-compose";
            delete params.viewId;
          }
          if (params.viewId) {
            name = "workspace-compose-view";
          }
          await router.push({
            name: name ?? undefined,
            params: {
              ...params,
              changeSetId,
            },
          });

          const statusStore = useStatusStore(changeSetId);
          statusStore.resetWhenChangingChangeset();
        },

        async FETCH_APPROVAL_STATUS(changeSetId: ChangeSetId) {
          return new ApiRequest<ApprovalData>({
            method: "get",
            url: BASE_API.concat([{ changeSetId }, "approval_status"]),
            onSuccess: (response) => {
              this.changeSetsApprovalData[changeSetId] = response;
            },
          });
        },

        async FETCH_CHANGE_SETS() {
          return new ApiRequest<WorkspaceMetadata>({
            method: "get",
            url: BASE_API,
            onSuccess: (response) => {
              this.headChangeSetId = response.defaultChangeSetId;
              this.changeSetsById = _.keyBy(response.changeSets, "id");
              this.defaultApprovers = response.approvers;
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
              this.changeSetsById[response.changeSet.id] = response.changeSet;
            },
          });
        },
        async ABANDON_CHANGE_SET() {
          if (this.creatingChangeSet)
            throw new Error("Wait until change set is created to abandon");
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          else if (this.headSelected) {
            throw new Error("You cannot abandon HEAD!");
          }
          if (
            router.currentRoute.value.name &&
            ["workspace-lab-packages", "workspace-lab-assets"].includes(
              router.currentRoute.value.name.toString(),
            )
          ) {
            router.push({ name: "workspace-lab" });
          }
          return new ApiRequest<{ changeSet: ChangeSet }>({
            method: "post",
            url: "change_set/abandon_change_set",
            params: {
              changeSetId: this.selectedChangeSet.id,
            },
            optimistic: () => {
              // remove component selections, its corrupting navigation
              const key = `${this.selectedChangeSetId}_selected_component`;
              window.localStorage.removeItem(key);
              const headkey = `${this.headChangeSetId}_selected_component`;
              window.localStorage.removeItem(headkey);
            },
            onSuccess: (response) => {
              // this.changeSetsById[response.changeSet.pk] = response.changeSet;
              const statusStore = useStatusStore();
              statusStore.resetWhenChangingChangeset();
            },
          });
        },
        async REQUEST_CHANGE_SET_APPROVAL() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          const selectedChangeSetId = this.selectedChangeSetId;
          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([{ selectedChangeSetId }, "request_approval"]),
          });
        },
        async APPROVE_CHANGE_SET_FOR_APPLY(id?: ChangeSetId) {
          const changeSetId = id || this.selectedChangeSetId;

          if (!changeSetId) throw new Error("Select a change set");

          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([{ changeSetId }, "approve"]),
            params: {
              status: "Approved",
            },
          });
        },
        async REJECT_CHANGE_SET_APPLY(id?: ChangeSetId) {
          const changeSetId = id || this.selectedChangeSetId;

          if (!changeSetId) throw new Error("Select a change set");

          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([{ changeSetId }, "approve"]),
            params: {
              status: "Rejected",
            },
          });
        },
        async CANCEL_APPROVAL_REQUEST() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          const selectedChangeSetId = this.selectedChangeSetId;
          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([
              { selectedChangeSetId },
              "cancel_approval_request",
            ]),
          });
        },
        async REOPEN_CHANGE_SET() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          const selectedChangeSetId = this.selectedChangeSetId;
          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([{ selectedChangeSetId }, "reopen"]),
          });
        },
        async APPLY_CHANGE_SET(username: string) {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          const selectedChangeSetId = this.selectedChangeSetId;

          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([{ selectedChangeSetId }, "apply"]),
            optimistic: () => {
              toast({
                component: IncomingChangesMerging,
                props: {
                  username,
                },
              });
            },
            _delay: 2000,
            onFail: (response) => {
              if (response.response.data.error.message) {
                toast(response.response.data.error.message, { timeout: 5000 });
              }
            },
          });
        },
        async BEGIN_APPROVAL_PROCESS() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest({
            method: "post",
            url: "change_set/begin_approval_process",
            params: {
              visibility_change_set_pk: this.selectedChangeSet.id,
            },
          });
        },
        async CANCEL_APPROVAL_PROCESS() {
          if (!this.selectedChangeSet) throw new Error("Select a change set");
          return new ApiRequest({
            method: "post",
            url: "change_set/cancel_approval_process",
            params: {
              visibility_change_set_pk: this.selectedChangeSet.id,
            },
          });
        },
        async RENAME_CHANGE_SET(changeSetId: ChangeSetId, newName: string) {
          return new ApiRequest({
            method: "post",
            url: BASE_API.concat([{ changeSetId }, "rename"]),
            params: { newName },
            optimistic: () => {
              const changeSet = this.changeSetsById[changeSetId];
              if (!changeSet) return;

              const oldName = changeSet.name;
              changeSet.name = newName;

              return () => {
                // if it fails, revert the name
                changeSet.name = oldName;
              };
            },
          });
        },

        getAutoSelectedChangeSetId() {
          const lastChangeSetId = sessionStorage.getItem(
            `SI:LAST_CHANGE_SET/${workspacePk}`,
          );
          if (
            lastChangeSetId &&
            this.changeSetsById[lastChangeSetId]?.status ===
              ChangeSetStatus.Open
          ) {
            return lastChangeSetId;
          }

          if (this.openChangeSets?.length <= 2) {
            // will select the single open change set or head if thats all that exists
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            return _.last(this.openChangeSets)!.id;
          }

          return this.headChangeSetId ?? false;
        },
        getGeneratedChangesetName() {
          let latestNum = 0;
          _.each(this.allChangeSets, (cs) => {
            const labelNum = Number(cs.name.split(" ").pop());
            if (!_.isNaN(labelNum) && labelNum > latestNum) {
              latestNum = labelNum;
            }
          });
          return `Change Set ${latestNum + 1}`;
        },

        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },
      async onActivated() {
        if (!workspacePk) return;
        await this.FETCH_CHANGE_SETS();
        const stopWatchSelectedChangeSet = watch(
          () => this.selectedChangeSet,
          () => {
            // store last used change set (per workspace) in localstorage
            if (this.selectedChangeSet && workspacePk) {
              sessionStorage.setItem(
                `SI:LAST_CHANGE_SET/${workspacePk}`,
                this.selectedChangeSet.id,
              );
            }
          },
          { immediate: true },
        );

        realtimeStore.subscribe(this.$id, `workspace/${workspacePk}`, [
          {
            eventType: "ChangeSetCreated",
            callback: this.FETCH_CHANGE_SETS,
          },
          {
            eventType: "ChangeSetStatusChanged",
            callback: async (data) => {
              if (
                [
                  ChangeSetStatus.Abandoned,
                  ChangeSetStatus.Applied,
                  ChangeSetStatus.Closed,
                ].includes(data.changeSet.status) &&
                data.changeSet.id !== this.headChangeSetId
              ) {
                if (featureFlagsStore.NEW_HOTNESS)
                  heimdall.prune(workspacePk, data.changeSet.id);
              }
              // If I'm the one who requested this change set - toast that it's been approved/rejected/etc.
              if (
                data.changeSet.mergeRequestedByUserId === authStore.user?.pk
              ) {
                if (data.changeSet.status === ChangeSetStatus.Rejected) {
                  toast({
                    component: ChangeSetStatusChanged,
                    props: {
                      user: data.changeSet.reviewedByUser,
                      command: "rejected the request to apply",
                      changeSetName: data.changeSet.name,
                    },
                  });
                } else if (data.changeSet.status === ChangeSetStatus.Approved) {
                  toast({
                    component: ChangeSetStatusChanged,
                    props: {
                      user: data.changeSet.reviewedByUser,
                      command: "approved the request to apply",
                      changeSetName: data.changeSet.name,
                    },
                  });
                }
              } else if (
                data.changeSet.status === ChangeSetStatus.NeedsApproval
              ) {
                this.FETCH_APPROVAL_STATUS(data.changeSet.id);
              }
              await this.FETCH_CHANGE_SETS();
            },
          },
          {
            eventType: "ChangeSetAbandoned",
            callback: async (data) => {
              if (data.changeSetId !== this.headChangeSetId) {
                if (featureFlagsStore.NEW_HOTNESS)
                  heimdall.prune(workspacePk, data.changeSetId);
              }

              const changeSetName = this.selectedChangeSet?.name;
              if (data.changeSetId === this.selectedChangeSetId) {
                if (this.headChangeSetId) {
                  await this.setActiveChangeset(this.headChangeSetId, false);

                  // TODO(Wendy) - move this logic out of the store when we can
                  // START REDIRECT FOR ABANDONED CHANGE SET IN NEWHOTNESS
                  const route = router.currentRoute.value;
                  const name = route.name;
                  if (name?.toString().startsWith("new-hotness")) {
                    const params = { ...route.params };
                    delete params.componentId;
                    await router.push({
                      name: "new-hotness-head",
                      params: {
                        ...params,
                        changeSetId: this.headChangeSetId,
                      },
                    });
                  }
                  // END REDIRECT FOR ABANDONED CHANGE SET IN NEWHOTNESS

                  toast({
                    component: MovedToHead,
                    props: {
                      icon: "trash",
                      changeSetName,
                      action: "abandoned",
                    },
                  });
                }
              }
              await this.FETCH_CHANGE_SETS();
            },
          },
          {
            eventType: "ChangeSetCancelled",
            callback: (data) => {
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              const { changeSetId, userPk } = data as any as {
                changeSetId: string;
                userPk: UserId;
              };
              const changeSet = this.changeSetsById[changeSetId];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Abandoned;
                if (this.selectedChangeSet?.id === changeSetId) {
                  this.postAbandonActor = userPk;
                }
                this.changeSetsById[changeSetId] = changeSet;
              }

              this.FETCH_CHANGE_SETS();
            },
          },
          {
            eventType: "ChangeSetApplied",
            callback: (data) => {
              const { changeSetId, userPk, toRebaseChangeSetId } = data;
              const changeSet = this.changeSetsById[changeSetId];
              if (changeSet) {
                if (changeSet.id !== this.headChangeSetId) {
                  // never set HEAD to Applied
                  changeSet.status = ChangeSetStatus.Applied;
                  if (featureFlagsStore.NEW_HOTNESS)
                    heimdall.prune(workspacePk, changeSet.id);
                }
                if (this.selectedChangeSet?.id === changeSetId) {
                  this.postApplyActor = userPk;
                }
                this.changeSetsById[changeSetId] = changeSet;
                // whenever the change set is applied move us to head
              }

              // TODO: jobelenus, I'm worried the WsEvent fires before commit happens
              /* if (this.selectedChangeSetId && !this.headSelected)
                this.FETCH_STATUS_WITH_BASE(this.selectedChangeSetId); */

              // did I get an update from head (and I am not head)?
              if (
                this.selectedChangeSetId === toRebaseChangeSetId &&
                this.selectedChangeSetId !== this.headChangeSetId
              ) {
                toast({
                  component: RebaseOnBase,
                });
              }

              // `list_open_change_sets` gets called prior on voters
              // which means the change set is gone, so always move
              if (
                !this.selectedChangeSetId ||
                this.selectedChangeSetId === changeSetId
              ) {
                const route = useRouterStore().currentRoute;

                if (route?.name) {
                  router.push({
                    name: route.name,
                    params: {
                      ...route.params,
                      changeSetId: "head",
                    },
                  });
                  if (
                    this.selectedChangeSet &&
                    this.selectedChangeSet.name !== "HEAD"
                  )
                    toast({
                      component: MovedToHead,
                      props: {
                        icon: "tools",
                        changeSetName: this.selectedChangeSet?.name,
                        action: "merged",
                      },
                    });
                }
              }
            },
          },
          {
            eventType: "ChangeSetBeginApprovalProcess",
            callback: (data) => {
              if (this.selectedChangeSet?.id === data.changeSetId) {
                this.changeSetApprovals = {};
              }
              const changeSet = this.changeSetsById[data.changeSetId];
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
              if (this.selectedChangeSet?.id === data.changeSetId) {
                this.changeSetApprovals = {};
              }
              const changeSet = this.changeSetsById[data.changeSetId];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Open;
              }
            },
          },

          {
            eventType: "ChangeSetBeginAbandonProcess",
            callback: (data) => {
              if (this.selectedChangeSet?.id === data.changeSetId) {
                this.changeSetApprovals = {};
              }
              const changeSet = this.changeSetsById[data.changeSetId];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.NeedsAbandonApproval;
                changeSet.abandonRequestedAt = new Date().toISOString();
                changeSet.abandonRequestedByUserId = data.userPk;
              }
            },
          },
          {
            eventType: "ChangeSetCancelAbandonProcess",
            callback: (data) => {
              if (this.selectedChangeSet?.id === data.changeSetId) {
                this.changeSetApprovals = {};
              }
              const changeSet = this.changeSetsById[data.changeSetId];
              if (changeSet) {
                changeSet.status = ChangeSetStatus.Open;
              }
            },
          },
          {
            eventType: "ChangeSetMergeVote",
            callback: (data) => {
              if (this.selectedChangeSet?.id === data.changeSetId) {
                this.changeSetApprovals[data.userPk] = data.vote;
              }
            },
          },
          {
            eventType: "ChangeSetAbandonVote",
            callback: (data) => {
              if (this.selectedChangeSet?.id === data.changeSetId) {
                this.changeSetApprovals[data.userPk] = data.vote;
              }
            },
          },
          {
            eventType: "ChangeSetWritten",
            callback: (changeSetId) => {
              this.changeSetsWrittenAtById[changeSetId] = new Date();
            },
          },
          {
            eventType: "ChangeSetApprovalStatusChanged",
            callback: (changeSetId) => {
              if (this.selectedChangeSet?.id === changeSetId) {
                this.FETCH_APPROVAL_STATUS(changeSetId);
              }
            },
          },
          {
            eventType: "ChangeSetRename",
            callback: ({ changeSetId, newName }) => {
              const changeSet = this.changeSetsById[changeSetId];
              if (changeSet) {
                changeSet.name = newName;
              }
            },
          },
        ]);

        const actionUnsub = this.$onAction(handleStoreError);

        return () => {
          actionUnsub();
          stopWatchSelectedChangeSet();
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
}
