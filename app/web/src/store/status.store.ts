import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { POSITION, useToast } from "vue-toastification";
import { watch } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import UpdatingModel from "../components/toasts/UpdatingModel.vue";

import handleStoreError from "./errors";

const GLOBAL_STATUS_TOAST_TIMEOUT = 300;
const GLOBAL_STATUS_TOAST_DEBOUNCE = 150;
export const GLOBAL_STATUS_TOAST_ID = "global_status_toast";

export type StatusMessageState = "statusStarted" | "statusFinished";

export interface DependentValuesUpdateStatusUpdate {
  kind: "dependentValueUpdate";
  status: StatusMessageState;
  componentId: string;
  timestamp: Date;
}

export interface RebaseStatusUpdate {
  kind: "rebase";
  status: StatusMessageState;
  timestamp: Date;
}

export type StatusUpdate = DependentValuesUpdateStatusUpdate | RebaseStatusUpdate;

export type GlobalUpdateStatus = {
  isUpdating: boolean;
};

export type ComponentUpdateStatus = {
  componentId: string;
  componentLabel: string;

  isUpdating: boolean;

  lastUpdateAt: Date;
  statusMessage: string;
};

export type AttributeValueId = string;

export interface AttributeValueStatus {
  componentId: string;
  startedAt: Date;
  finishedAt?: Date;
}

export type ValueIsForKind = "prop" | "inputSocket" | "outputSocket";
export interface ValueIsFor {
  kind: ValueIsForKind;
  id?: string;
}

export type ComponentStatusDetails = {
  lastUpdatedAt?: Date;
  message: string;
};

export interface RebaseStatus {
  rebaseStart?: Date;
  rebaseFinished?: Date;
  count: number;
}

export type Conflict = string;

export interface StatusStoreState {
  activeComponents: Record<ComponentId, DependentValuesUpdateStatusUpdate>;
  dvuRootsCount: number;
  rebaseStatus: RebaseStatus;
}

const FIFTEEN_SECONDS_MS = 1000 * 15;

export const useStatusStore = (forceChangeSetId?: ChangeSetId) => {
  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | undefined;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    const changeSetsStore = useChangeSetsStore();
    changeSetId = changeSetsStore.selectedChangeSetId;
  }

  const realtimeStore = useRealtimeStore();
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const toast = useToast();

  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/status`, {
      state: (): StatusStoreState => ({
        activeComponents: {},
        rebaseStatus: { count: 0 },
        dvuRootsCount: 0,
      }),
      getters: {
        globalStatus(state): GlobalUpdateStatus {
          const nowMs = Date.now();
          const isUpdatingDvus = Object.keys(this.activeComponents).length > 0 || this.dvuRootsCount > 0;

          const isRebasing =
            !state.rebaseStatus.rebaseFinished && nowMs - Number(state.rebaseStatus.rebaseStart) > FIFTEEN_SECONDS_MS;

          return {
            isUpdating: isUpdatingDvus || isRebasing,
          };
        },
        globalStatusMessage(): string {
          if (this.globalStatus.isUpdating) {
            return "Updating the model";
          }
          return "Model is up to date";
        },
        componentIsLoading: (state) => (id: ComponentId) => {
          return state.activeComponents[id] !== undefined;
        },
      },
      actions: {
        resetWhenChangingChangeset() {
          this.activeComponents = {};
          this.rebaseStatus = {
            rebaseStart: undefined,
            rebaseFinished: undefined,
            count: 0,
          };
          this.dvuRootsCount = 0;
        },
        async FETCH_DVU_ROOTS() {
          return new ApiRequest<{ count: number }>({
            url: "diagram/dvu_roots",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.dvuRootsCount = response.count;
              if (response.count < 1) {
                this.activeComponents = {};
              }
            },
          });
        },

        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },
      onActivated() {
        if (!changeSetId) return;

        const debouncedFetchDvuRoots = _.debounce(this.FETCH_DVU_ROOTS, 1000);

        // Just in case we miss a change set written and we still think there
        // are roots, but we don't know about any active components.
        const dvuRootCheck = setInterval(async () => {
          if (this.dvuRootsCount > 0 && Object.keys(this.activeComponents).length < 1) {
            const resp = await debouncedFetchDvuRoots();
            if (!resp?.result.success) {
              clearInterval(dvuRootCheck);
            }
          }
        }, 3500);

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "ChangeSetWritten",
            callback: () => {
              if (this.dvuRootsCount > 0) {
                debouncedFetchDvuRoots();
              }
            },
          },
          {
            eventType: "StatusUpdate",
            callback: (update, _metadata) => {
              if (!this.globalStatus.isUpdating) {
                // if we're done updating, clear the old data
                this.activeComponents = {};
                this.rebaseStatus = {
                  rebaseStart: undefined,
                  rebaseFinished: undefined,
                  count: 0,
                };
              }

              if (update.kind === "dependentValueUpdate") {
                if (update.status === "statusStarted") {
                  this.activeComponents[update.componentId] = update;
                } else if (update.status === "statusFinished") {
                  if (this.activeComponents[update.componentId] !== undefined) {
                    delete this.activeComponents[update.componentId];
                  }
                }
              } else if (update.kind === "rebase") {
                if (update.status === "statusStarted") {
                  if (!this.rebaseStatus.rebaseStart) {
                    this.rebaseStatus.rebaseStart = update.timestamp;
                  }

                  this.rebaseStatus.count++;
                } else if (update.status === "statusFinished") {
                  if (this.rebaseStatus.rebaseStart) {
                    this.rebaseStatus.count--;
                    if (this.rebaseStatus.count <= 0) {
                      this.rebaseStatus.rebaseFinished = update.timestamp;
                    }
                  }
                }
              }
            },
          },
        ]);

        // This watcher updates the GlobalStatus toast
        watch(
          () => this.globalStatus.isUpdating,
          () => {
            _.debounce(
              () => {
                toast.update(
                  GLOBAL_STATUS_TOAST_ID,
                  {
                    content: {
                      component: UpdatingModel,
                    },
                    options: {
                      position: POSITION.TOP_CENTER,
                      timeout: this.globalStatus.isUpdating ? false : GLOBAL_STATUS_TOAST_TIMEOUT,
                      closeOnClick: !this.globalStatus.isUpdating,
                      toastClassName: "si-toast-no-defaults",
                    },
                  },
                  true,
                );
              },
              GLOBAL_STATUS_TOAST_DEBOUNCE,
              { leading: true },
            )();
          },
        );

        const actionUnsub = this.$onAction(handleStoreError);

        return () => {
          actionUnsub();
          clearInterval(dvuRootCheck);
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
