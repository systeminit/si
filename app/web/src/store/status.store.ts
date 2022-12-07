import { defineStore } from "pinia";
import _ from "lodash";
import async from "async";

import { addStoreHooks } from "@/utils/pinia_hooks_plugin";

import promiseDelay from "@/utils/promise_delay";
import { ChangeSetId, useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";

import { ComponentId } from "./components.store";

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

  isUpdating: boolean; // note - might change to enum if more states appear

  stepsCountCurrent: number;
  stepsCountTotal: number;

  statusMessage: string; // ex: updating attributes

  lastStepCompletedAt: Date;

  byActor?:
    | { type: "system" }
    | {
        type: "user";
        id: string;
        label: string;
      };
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
        globalStatus: {} as Partial<GlobalUpdateStatus> | null,
        componentStatusById: {} as Record<ComponentId, ComponentUpdateStatus>,
      }),
      getters: {
        latestComponentUpdate(state) {
          const sortedUpdates = _.orderBy(
            _.values(state.componentStatusById),
            (cu) => cu.lastStepCompletedAt,
          );
          return sortedUpdates.pop();
        },
      },
      actions: {
        async FETCH_CURRENT_STATUS() {
          this.globalStatus = {
            isUpdating: false,
          };

          // return new ApiRequest<{
          //   global: GlobalUpdateStatus;
          //   components: Record<ComponentId, ComponentUpdateStatus>;
          // }>({
          //   url: "/status",
          //   // TODO: do we want to pass these through as headers? or in URL?
          //   params: {
          //     workspaceId,
          //     changeSetId,
          //   },
          //   onSuccess: (response) => {
          //     this.globalStatus = response.global;
          //     this.componentStatusById = _.keyBy(response.components, "id");
          //   },
          // });
        },

        async triggerMockUpdateFlow(componentIds: ComponentId[]) {
          const realtimeStore = useRealtimeStore();
          const updateStartedAt = new Date();

          const STEPS_PER_COMPONENT = 3;

          let i = 0;
          let componentsCountCurrent = 0;
          const componentsCountTotal = componentIds.length;
          const stepsCountTotal = STEPS_PER_COMPONENT * componentsCountTotal;
          let stepsCountCurrent = 0;
          // kick these off in parallel, with an initial delay to make the steps happen in series-ish
          await async.each(componentIds, async (componentId) => {
            await promiseDelay(i++ * 250);
            let componentStepsCountCurrent = 0;
            let componentComplete = false;

            for (const step of [
              {
                message: "Updating attributes",
                delay: 500,
                stepComplete: false,
              },
              { message: "Attributes updated", delay: 50, stepComplete: true },
              { message: "Generating code", delay: 2000, stepComplete: false },
              { message: "Code generated", delay: 50, stepComplete: true },
              {
                message: "Running qualifications",
                delay: 4000,
                complete: false,
              },
              { message: "Component updated", delay: 0, stepComplete: true },
            ]) {
              if (step.stepComplete) {
                stepsCountCurrent++;
                componentStepsCountCurrent++;
                if (componentStepsCountCurrent === STEPS_PER_COMPONENT) {
                  componentComplete = true;
                  componentsCountCurrent++;
                }
              }

              const currentUpdateAt = new Date();

              realtimeStore.mockEvent("UpdateStatus", {
                global: {
                  isUpdating: componentsCountCurrent < componentsCountTotal,
                  componentsCountCurrent,
                  componentsCountTotal,
                  stepsCountCurrent,
                  stepsCountTotal,
                  updateStartedAt, // NOTE - will probably be coming through as strings and need conversion...
                  lastStepCompletedAt: currentUpdateAt,
                },
                components: [
                  {
                    componentId,
                    statusMessage: step.message,
                    isUpdating: !componentComplete,
                    lastStepCompletedAt: currentUpdateAt,
                    stepsCountCurrent: componentStepsCountCurrent,
                    stepsCountTotal: STEPS_PER_COMPONENT,
                    byActor: {
                      type: "user",
                      label: "theo",
                      id: 100,
                    },
                  },
                ],
              });

              await promiseDelay(step.delay + step.delay * Math.random());
            }
          });
        },
      },
      onActivated() {
        if (!changeSetId) return;

        this.FETCH_CURRENT_STATUS();

        const realtimeStore = useRealtimeStore();

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "UpdateStatus",
            callback: (update) => {
              this.globalStatus = update.global;
              if (update.components) {
                _.each(update.components, (cu) => {
                  this.componentStatusById[cu.componentId] = cu;
                });
              }
            },
          },
        ]);

        return () => {
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
