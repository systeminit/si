import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { POSITION, useToast } from "vue-toastification";
import { watch } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import UpdatingModel from "../components/toasts/UpdatingModel.vue";

import handleStoreError from "./errors";

const GLOBAL_STATUS_TOAST_TIMEOUT = 1000;
const GLOBAL_STATUS_TOAST_DEBOUNCE = 300;
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

export type StatusUpdate =
  | DependentValuesUpdateStatusUpdate
  | RebaseStatusUpdate;

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
  rebaseStatus: RebaseStatus;
}

const FIFTEEN_SECONDS_MS = 1000 * 15;
const TEN_MINUTES_MS = 1000 * 60 * 10;

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

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const toast = useToast();

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/status`,
      {
        state: (): StatusStoreState => ({
          activeComponents: {},
          rebaseStatus: { count: 0 },
        }),
        getters: {
          globalStatus(state): GlobalUpdateStatus {
            const nowMs = Date.now();
            const isUpdatingDvus = _.some(state.activeComponents, (status) => {
              const startedAt = Number(status.timestamp);
              // don't consider an update live after 10 minutes. Instead of this
              // we should ask the graph whether it still has dependent value
              // updates...
              if (nowMs - startedAt > TEN_MINUTES_MS) {
                return false;
              }

              return true;
            });

            const isRebasing =
              !state.rebaseStatus.rebaseFinished &&
              nowMs - Number(state.rebaseStatus.rebaseStart) >
                FIFTEEN_SECONDS_MS;

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
        onActivated() {
          if (!changeSetId) return;

          const realtimeStore = useRealtimeStore();
          let cleanupTimeout: Timeout;

          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
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
                    if (
                      this.activeComponents[update.componentId] !== undefined
                    ) {
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
                        timeout: this.globalStatus.isUpdating
                          ? false
                          : GLOBAL_STATUS_TOAST_TIMEOUT,
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
            clearTimeout(cleanupTimeout);
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
