import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { ChangeSetId, useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";

import { ComponentId, SocketId, useComponentsStore } from "./components.store";

// NOTE - some uncertainty around transition from update finished state ("5/5 update complete") back to idle ("Model is up to date")
export type GlobalUpdateStatus = {
  // NOTE - might want an state enum here as well (for example to turn the bar into an error state)

  isUpdating: boolean;

  stepsCountCurrent: number;
  stepsCountTotal: number;

  componentsCountCurrent: number;
  componentsCountTotal: number;

  // not loving these names...
  updateStartedAt: Date; // timestamp when this update/batch was kicked off
  lastStepCompletedAt: Date; // timestamp of latest processed update within this cascade of updates
};

export type ComponentUpdateStatus = {
  componentId: string;
  componentLabel: string;

  isUpdating: boolean; // note - might change to enum if more states appear

  stepsCountCurrent: number;
  stepsCountTotal: number;

  statusMessage: string; // ex: updating attributes

  lastUpdateAt: Date;
  lastUpdateBy?: ActorView;
};

export type StatusUpdatePk = string;

export type AttributeValueStatus = "queued" | "running" | "completed";

export type AttributeValueId = string;

export type UpdateStatusTimestamps = {
  queuedAt: Date;
  runningAt?: Date;
  completedAt?: Date;
};

export type AttributeValueKind =
  | "internal"
  | "attribute"
  | "codeGen"
  | "qualification"
  | "inputSocket"
  | "outputSocket";

export type ComponentStatusDetails = {
  lastUpdatedAt?: Date;
  valueKindByValueId: Record<
    AttributeValueId,
    { kind: AttributeValueKind; id?: string }
  >;
  message: string;
  timestampsByValueId: Record<AttributeValueId, UpdateStatusTimestamps>;
};

export const useStatusStore = (forceChangeSetId?: ChangeSetId) => {
  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | null;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    const changeSetsStore = useChangeSetsStore();
    changeSetId = changeSetsStore.selectedChangeSetId;
  }

  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/status`, {
      state: () => ({
        calculatingUpdateSize: false,
        updateMetadataByPk: {} as Record<
          StatusUpdatePk,
          {
            actor: ActorView;
          }
        >,
        rawStatusesByValueId: {} as Record<
          AttributeValueId,
          {
            valueId: AttributeValueId;
            valueKind: { kind: AttributeValueKind; id?: string };
            componentId: ComponentId;
            statusTimestampsByUpdatePk: Record<
              StatusUpdatePk,
              UpdateStatusTimestamps
            >;
          }
        >,
      }),
      getters: {
        getSocketStatus:
          (state) => (componentId: ComponentId, socketId: SocketId) => {
            const valueId = _.findKey(
              state.rawStatusesByValueId,
              (valueMetadata) =>
                valueMetadata.componentId === componentId &&
                valueMetadata.valueKind.kind.endsWith("Socket") &&
                valueMetadata.valueKind.id === socketId,
            );
            if (!valueId) return "idle";
            const timestamps =
              state.rawStatusesByValueId[valueId]?.statusTimestampsByUpdatePk;
            if (!timestamps) return "idle";
            if (_.some(timestamps, (ts) => !ts.completedAt && !ts.runningAt)) {
              return "queued";
            } else if (_.some(timestamps, (ts) => !ts.completedAt)) {
              return "running";
            } else {
              return "completed";
            }
          },

        // helper to condense value timestamps down to a single status
        // statusesByValueId: (state) => {
        //   return _.mapValues(state.rawStatusesByValueId, (rawStatus) => {
        //     let status: AttributeValueStatus;
        //     const timestampsArray = _.values(
        //       rawStatus.statusTimestampsByUpdatePk,
        //     );
        //     if (
        //       _.some(
        //         timestampsArray,
        //         (ts) => ts.queuedAt && !ts.runningAt && !ts.completedAt,
        //       )
        //     ) {
        //       status = "queued";
        //     } else if (
        //       _.some(
        //         timestampsArray,
        //         (ts) => ts.queuedAt && ts.runningAt && !ts.completedAt,
        //       )
        //     ) {
        //       status = "running";
        //     } else {
        //       status = "completed";
        //     }

        //     return {
        //       valueKind: rawStatus.valueKind,
        //       componentId: rawStatus.componentId,
        //       status,
        //     };
        //   });
        // },

        componentStatusById(): Record<ComponentId, ComponentUpdateStatus> {
          const valueStatusesGroupedByComponentId = _.groupBy(
            this.rawStatusesByValueId,
            (valueStatus) => valueStatus.componentId,
          );

          const componentsStore = useComponentsStore();
          return _.mapValues(componentsStore.componentsById, (component) => {
            const valueStatuses =
              valueStatusesGroupedByComponentId[component.id];
            const componentLabel = `${component.schemaName} '${component.displayName}'`;

            // creates a dummy status entry for all components in the changeset
            // using timestamps from the list components endpoint
            if (!valueStatuses) {
              return {
                componentId: component.id,
                componentLabel,
                isUpdating: false,
                stepsCountCurrent: 0,
                stepsCountTotal: 0,
                lastUpdateAt: new Date(component.updatedInfo.timestamp),
                lastUpdateBy: component.updatedInfo.actor,
                statusMessage: "Component updated",
              };
            }

            let stepsCountCurrent = 0;
            let stepsCountTotal = 0;
            let isUpdating = false;

            // start with date in past
            let latestChangedTimestamp = new Date(0);
            let latestChangedValueId = null;
            let latestUpdatePk: StatusUpdatePk | null = null;

            _.each(valueStatuses, (vs) => {
              _.each(vs.statusTimestampsByUpdatePk, (ts, updatePk) => {
                stepsCountTotal++;

                if (ts.queuedAt && !ts.runningAt) {
                  // queued
                  isUpdating = true;
                } else if (ts.runningAt && !ts.completedAt) {
                  // running
                  isUpdating = true;
                  if (
                    vs.valueKind.kind !== "internal" &&
                    ts.runningAt > latestChangedTimestamp
                  ) {
                    latestChangedTimestamp = ts.runningAt;
                    latestChangedValueId = vs.valueId;
                    latestUpdatePk = updatePk;
                  }
                } else if (ts.completedAt) {
                  // completed
                  stepsCountCurrent++;
                  if (
                    vs.valueKind.kind !== "internal" &&
                    ts.completedAt > latestChangedTimestamp
                  ) {
                    latestChangedTimestamp = ts.completedAt;
                    latestChangedValueId = vs.valueId;
                    latestUpdatePk = updatePk;
                  }
                }
              });
            });

            let statusMessage = "Updating component";
            if (latestChangedValueId) {
              const valueKind =
                this.rawStatusesByValueId[latestChangedValueId]?.valueKind.kind;
              if (valueKind) {
                statusMessage = {
                  codeGen: "Running code gen",
                  attribute: "Updating attributes",
                  qualification: "Running qualifications",
                  inputSocket: "Updating input socket values",
                  outputSocket: "Updating output socket values",
                  internal: "Updating internal wiring",
                }[valueKind];
              }
            }

            if (!isUpdating) statusMessage = "Component updated";

            return {
              componentId: component.id,
              componentLabel,
              isUpdating,
              stepsCountCurrent,
              stepsCountTotal,
              statusMessage,
              lastUpdateAt: latestChangedTimestamp,
              ...(latestUpdatePk !== null && {
                lastUpdateBy: this.updateMetadataByPk[latestUpdatePk]?.actor,
              }),
            };
          });
        },

        latestComponentUpdate(): ComponentUpdateStatus | undefined {
          const sortedUpdates = _.orderBy(
            _.values(this.componentStatusById),
            (cu) => cu.lastUpdateAt,
          );
          return sortedUpdates.pop();
        },
        globalStatus(): GlobalUpdateStatus {
          const isUpdating = _.some(
            this.componentStatusById,
            (status) => status.isUpdating,
          );
          const stepsCountCurrent = _.sumBy(
            _.values(this.componentStatusById),
            (status) => status.stepsCountCurrent,
          );
          const stepsCountTotal = _.sumBy(
            _.values(this.componentStatusById),
            (status) => status.stepsCountTotal,
          );

          const componentsCountCurrent = _.filter(
            this.componentStatusById,
            (status) => status.stepsCountTotal > 0 && !status.isUpdating,
          ).length;

          // we now have a fake component status for each component, even when no updates are happening
          // so we must filter for those with some "steps" (value updates)
          const componentsCountTotal = _.filter(
            this.componentStatusById,
            (cs) => cs.stepsCountTotal > 0,
          ).length;

          // handle special case for when update just began but we have not gotten details from backend yet
          if (this.calculatingUpdateSize && !isUpdating) {
            return {
              isUpdating: true,
              stepsCountCurrent: 1,
              stepsCountTotal: 100,
              componentsCountCurrent: 0,
              componentsCountTotal: Infinity,
              // TODO(wendy) - can we remove these?
              updateStartedAt: new Date(),
              lastStepCompletedAt: new Date(),
            };
          }

          return {
            isUpdating,
            stepsCountCurrent,
            stepsCountTotal,
            componentsCountCurrent,
            componentsCountTotal,
            // TODO(wendy) - can we remove these?
            updateStartedAt: new Date(),
            lastStepCompletedAt: new Date(),
          };
        },
        globalStatusMessage(): string {
          if (this.globalStatus.isUpdating || this.calculatingUpdateSize) {
            return "Updating & testing the model";
          }
          return "Model is up to date";
        },
        globalStatusDetailMessage(): string | undefined {
          if (this.calculatingUpdateSize) return "Calculating scope of update";
          if (!this.globalStatus.isUpdating) return;
          const latestUpdate = this.latestComponentUpdate;
          if (!latestUpdate) return;
          return `${latestUpdate.statusMessage} - ${latestUpdate.componentLabel}`;
        },
      },
      actions: {
        async FETCH_CURRENT_STATUS() {
          return new ApiRequest<
            {
              pk: StatusUpdatePk;
              data: {
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                actor: ActorView;
                attributeValueId: AttributeValueId; // id of attribute that kicked off the update
                dependendValuesMetadata: Record<
                  AttributeValueId,
                  {
                    valuedId: AttributeValueId;
                    componentId: ComponentId;
                    valueKind: { kind: AttributeValueKind; id?: string };
                  }
                >;
                queuedDependentValueIds: AttributeValueId[];
                runningDependentValueIds: AttributeValueId[];
                completedDependentValueIds: AttributeValueId[];
              };
            }[]
          >({
            url: "status/list-active-statuses",
            params: {
              changeSetPk: changeSetId,
            },
            onSuccess: (allUpdates) => {
              const now = new Date();
              // can have multiple updates in progress
              _.each(allUpdates, (singleUpdate) => {
                // record some info about the update itself
                this.updateMetadataByPk[singleUpdate.pk] = {
                  actor: singleUpdate.data.actor,
                };

                // fill in data for each value
                _.each(
                  singleUpdate.data.dependendValuesMetadata,
                  (valueMetadata, valueId) => {
                    const {
                      runningDependentValueIds,
                      completedDependentValueIds,
                    } = singleUpdate.data;

                    const rawStatus = this.rawStatusesByValueId[valueId] ?? {
                      valueId,
                      valueKind: valueMetadata.valueKind,
                      componentId: valueMetadata.componentId,
                      statusTimestampsByUpdatePk: {},
                    };

                    // use fake timestamps based on their status
                    const timestamps = {
                      queuedAt: now,
                      ...(runningDependentValueIds.includes(valueId) && {
                        runningAt: now,
                      }),
                      ...(completedDependentValueIds.includes(valueId) && {
                        completedAt: now,
                      }),
                    };
                    rawStatus.statusTimestampsByUpdatePk[singleUpdate.pk] =
                      timestamps;
                    this.rawStatusesByValueId[valueId] = rawStatus;
                  },
                );
              });
            },
          });
        },

        markUpdateStarted() {
          this.calculatingUpdateSize = true;
        },
        cancelUpdateStarted() {
          this.calculatingUpdateSize = false;
        },

        checkCompletedCleanup() {
          if (!this.globalStatus.isUpdating) {
            // if we're done updating, clear the timestamps
            this.rawStatusesByValueId = {};
          }
        },
      },
      onActivated() {
        if (!changeSetId) return;

        this.FETCH_CURRENT_STATUS();

        const realtimeStore = useRealtimeStore();
        let cleanupTimeout: Timeout;

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "StatusUpdate",
            callback: (update, _metadata) => {
              if (!update?.pk) {
                return;
              }

              // fill in update metadata if this the first time we're seeing this specific update
              if (!this.updateMetadataByPk[update.pk]) {
                this.updateMetadataByPk[update.pk] = { actor: update.actor };
              }

              if (update.status === "statusStarted") {
                // not sure if we need to do anything?
                return;
              } else if (update.status === "statusFinished") {
                if (cleanupTimeout) {
                  clearTimeout(cleanupTimeout);
                }
                cleanupTimeout = setTimeout(this.checkCompletedCleanup, 2000);
                return;
              }

              const now = new Date();
              update.values.forEach(({ componentId, valueId, valueKind }) => {
                const valueStatusData = this.rawStatusesByValueId[valueId] ?? {
                  valueId,
                  valueKind,
                  componentId,
                  statusTimestampsByUpdatePk: {
                    [update.pk]: {
                      queuedAt: now,
                    },
                  },
                };

                const statusUpdate = valueStatusData.statusTimestampsByUpdatePk[
                  update.pk
                ] ?? {
                  queuedAt: now,
                };

                switch (update.status) {
                  case "completed":
                    statusUpdate.completedAt = now;
                    statusUpdate.runningAt ||= now;
                    break;
                  case "running":
                    statusUpdate.runningAt = now;
                    break;
                  case "queued":
                  default:
                    statusUpdate.runningAt = undefined;
                    statusUpdate.completedAt = undefined;
                    statusUpdate.runningAt = now;
                    break;
                }

                this.rawStatusesByValueId[valueId] = {
                  ...valueStatusData,
                  statusTimestampsByUpdatePk: {
                    ...valueStatusData.statusTimestampsByUpdatePk,
                    [update.pk]: statusUpdate,
                  },
                };
              });
              // if we are receiving the queued event, we'll clear our locally stored loading state we set when the attribute was updated
              if (update.status === "queued" && this.calculatingUpdateSize) {
                this.calculatingUpdateSize = false;
              }
            },
          },
        ]);

        return () => {
          clearTimeout(cleanupTimeout);
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
