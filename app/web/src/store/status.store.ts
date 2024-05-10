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

import { SocketId, useComponentsStore } from "./components.store";

const GLOBAL_STATUS_TOAST_TIMEOUT = 1000;
const GLOBAL_STATUS_TOAST_DEBOUNCE = 300;
export const GLOBAL_STATUS_TOAST_ID = "global_status_toast";

export type StatusMessageState = "statusStarted" | "statusFinished";

export interface DependentValuesUpdateStatusUpdate {
  kind: "dependentValueUpdate";
  status: StatusMessageState;
  valueId: string;
  componentId: string;
  isFor: ValueIsFor;
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
  timeouts: AttributeValueStatus[];

  updatedComponents: number;
  // This is not all the components in the graph, but all the components in the in-flight updates
  totalComponents: number;
};

export type ComponentUpdateStatus = {
  componentId: string;
  componentLabel: string;

  isUpdating: boolean;

  lastUpdateAt: Date;
  statusMessage: string;
  // lastUpdateBy?: ActorView;
};

export type AttributeValueId = string;

export interface AttributeValueStatus {
  valueId: AttributeValueId;
  componentId: string;
  isFor: ValueIsFor;
  count: number;
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
  valueIsforByValueId: Record<AttributeValueId, ValueIsFor>;
  message: string;
};

export interface RebaseStatus {
  rebaseStart?: Date;
  rebaseFinished?: Date;
  count: number;
}

export interface StatusStoreState {
  rawStatusesByValueId: Record<AttributeValueId, AttributeValueStatus>;
  rebaseStatus: RebaseStatus;
}

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
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/status`,
      {
        state: (): StatusStoreState => ({
          rawStatusesByValueId: {},
          rebaseStatus: { count: 0 },
        }),
        getters: {
          getSocketStatus:
            (state) => (componentId: ComponentId, socketId: SocketId) => {
              const valueId = _.findKey(
                state.rawStatusesByValueId,
                (valueMetadata) =>
                  valueMetadata.componentId === componentId &&
                  valueMetadata.isFor.kind.endsWith("Socket") &&
                  valueMetadata.isFor.id === socketId,
              );
              if (!valueId) return "idle";
              const startedAt = state.rawStatusesByValueId[valueId]?.startedAt;
              const finishedAt =
                state.rawStatusesByValueId[valueId]?.finishedAt;

              if (!(startedAt || finishedAt)) return "idle";
              if (!finishedAt) {
                return "running";
              } else {
                return "completed";
              }
            },

          getComponentStatus:
            (state) =>
            (componentId: ComponentId): ComponentUpdateStatus | undefined => {
              const statuses = Object.values(state.rawStatusesByValueId).filter(
                (status) => status.componentId === componentId,
              );
              if (statuses.length === 0) {
                return undefined;
              }

              const componentStore = useComponentsStore();
              const component = componentStore.componentsById[componentId];
              if (!component) {
                return undefined;
              }

              const componentStatus: ComponentUpdateStatus = {
                componentId,
                isUpdating: true,
                lastUpdateAt: new Date(0),
                componentLabel: `${component.displayName} (${component.schemaName})`,
                statusMessage: "",
              };

              const isUpdating =
                statuses.filter((status) => !status.finishedAt).length > 0;
              componentStatus.isUpdating = isUpdating;
              if (isUpdating) {
                componentStatus.statusMessage = "Updating";
              } else {
                componentStatus.statusMessage = "";
              }

              for (const status of statuses) {
                if (status.finishedAt) {
                  componentStatus.lastUpdateAt =
                    status.finishedAt > componentStatus.lastUpdateAt
                      ? status.finishedAt
                      : componentStatus.lastUpdateAt;
                }
              }

              return componentStatus;
            },

          latestComponentUpdate(): ComponentUpdateStatus | undefined {
            const sortedUpdates = _.orderBy(
              _.values(this.rawStatusesByValueId),
              (cu) => cu.finishedAt,
            );
            const componentId = sortedUpdates.pop()?.componentId;

            if (componentId) {
              return this.getComponentStatus(componentId);
            }
          },

          globalStatus(state): GlobalUpdateStatus {
            const nowMs = Date.now();
            const FIFTEEN_SECONDS_MS = 1000 * 15;
            const timeouts = [] as AttributeValueStatus[];
            const isUpdatingDvus = _.some(
              state.rawStatusesByValueId,
              (status) => {
                const startedAt = Number(status.startedAt);
                // don't consider an update live after 15 seconds
                if (nowMs - startedAt > FIFTEEN_SECONDS_MS) {
                  // We could emit an error here for the attribute value or the
                  // component
                  timeouts.push(status);
                  return false;
                }

                return !status.finishedAt;
              },
            );

            const isRebasing =
              !state.rebaseStatus.rebaseFinished &&
              nowMs - Number(state.rebaseStatus.rebaseStart) >
                FIFTEEN_SECONDS_MS;

            const updatedComponents = _.keys(
              _.groupBy(
                _.filter(
                  state.rawStatusesByValueId,
                  (status) => !!status.finishedAt,
                ),
                (status) => status.componentId,
              ),
            ).length;

            const totalComponents = _.keys(
              _.groupBy(
                state.rawStatusesByValueId,
                (status) => status.componentId,
              ),
            ).length;

            return {
              isUpdating: isUpdatingDvus || isRebasing,
              timeouts,
              updatedComponents,
              totalComponents,
            };
          },
          globalStatusMessage(): string {
            if (this.globalStatus.isUpdating) {
              return "Updating the model";
            }
            return "Model is up to date";
          },
          globalStatusDetailMessage(): string | undefined {
            if (!this.globalStatus.isUpdating) return;
            const latestUpdate = this.latestComponentUpdate;
            if (!latestUpdate) return;
            return `Updating ${latestUpdate.componentLabel}`;
          },
        },
        onActivated() {
          if (!changeSetId) return;

          // this.FETCH_CURRENT_STATUS();

          const realtimeStore = useRealtimeStore();
          let cleanupTimeout: Timeout;

          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "StatusUpdate",
              callback: (update, _metadata) => {
                if (!this.globalStatus.isUpdating) {
                  // if we're done updating, clear the old data
                  this.rawStatusesByValueId = {};
                  this.rebaseStatus = {
                    rebaseStart: undefined,
                    rebaseFinished: undefined,
                    count: 0,
                  };
                }

                if (update.kind === "dependentValueUpdate") {
                  let currentStatusForValue =
                    this.rawStatusesByValueId[update.valueId];
                  if (update.status === "statusStarted") {
                    if (
                      !currentStatusForValue ||
                      currentStatusForValue.finishedAt !== undefined
                    ) {
                      currentStatusForValue = {
                        valueId: update.valueId,
                        componentId: update.componentId,
                        isFor: update.isFor,
                        startedAt: update.timestamp,
                        count: 0,
                      };
                    }

                    currentStatusForValue.count++;
                  } else if (update.status === "statusFinished") {
                    // If we get a finished message for a value we didn't get a
                    // start message, just ignore it
                    if (currentStatusForValue) {
                      currentStatusForValue.count--;
                      if (currentStatusForValue.count <= 0) {
                        currentStatusForValue.finishedAt = update.timestamp;
                      }
                    }
                  }

                  if (currentStatusForValue) {
                    this.rawStatusesByValueId[update.valueId] =
                      currentStatusForValue;
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
                  const timeout = this.globalStatus.timeouts.length > 0;

                  toast.update(
                    GLOBAL_STATUS_TOAST_ID,
                    {
                      content: {
                        component: UpdatingModel,
                        props: {
                          timeout,
                        },
                      },
                      options: {
                        position: POSITION.TOP_CENTER,
                        timeout:
                          this.globalStatus.isUpdating || timeout
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

          return () => {
            clearTimeout(cleanupTimeout);
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
