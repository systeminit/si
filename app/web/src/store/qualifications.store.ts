import { defineStore } from "pinia";
import _ from "lodash";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { Qualification } from "@/api/sdf/dal/qualification";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { ComponentId } from "./components.store";

export type QualificationStatus = "success" | "failure" | "running";

// TODO: align these key names with the status (ex: succeeded -> success)
type QualificationStats = {
  total: number;
  succeeded: number;
  failed: number;
  running: number;
};

export const useQualificationsStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const workspaceId = changeSetsStore.selectedWorkspaceId;
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/qualifications`, {
      state: () => ({
        // stats per component - this is the raw data
        // we may change this to store qualificaiton ids and individual statuses to make realtime updates easier
        qualificationStatsByComponentId: {} as Record<
          ComponentId,
          QualificationStats
        >,

        qualificationsByComponentId: {} as Record<ComponentId, Qualification[]>,

        checkedQualificationsAt: null as Date | null,
      }),
      getters: {
        // single status per component
        qualificationStatusByComponentId(): Record<
          ComponentId,
          QualificationStatus
        > {
          return _.mapValues(this.qualificationStatsByComponentId, (cs) => {
            if (cs.running) return "running";
            if (cs.failed > 0) return "failure";
            return "success";
          });
        },

        // stats/totals by component
        componentStats(): Record<QualificationStatus | "total", number> {
          const grouped = _.groupBy(this.qualificationStatusByComponentId);
          return {
            failure: grouped.failure?.length || 0,
            success: grouped.success?.length || 0,
            running: grouped.running?.length || 0,
            total: _.keys(this.qualificationStatusByComponentId).length,
          };
        },

        // roll up to single status for the workspace
        overallStatus(): QualificationStatus {
          if (this.componentStats.running > 0) return "running";
          if (this.componentStats.failure > 0) return "failure";
          return "success";
        },
      },
      actions: {
        async FETCH_QUALIFICATIONS_SUMMARY() {
          return new ApiRequest<{
            total: number;
            succeeded: number;
            failed: number;
            components: {
              componentId: string;
              componentName: string;
              total: number;
              succeeded: number;
              failed: number;
            }[];
          }>({
            url: "qualification/get_summary",
            params: {
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (response) => {
              // response also includes component totals, but we'll ignore it and use getters instead
              const byComponentId = _.keyBy(response.components, "componentId");
              this.qualificationStatsByComponentId = _.mapValues(
                byComponentId,
                (cs) => {
                  return {
                    // transform the data slightly to add "running" so we can avoid recalculating again elsewhere
                    total: cs.total,
                    succeeded: cs.succeeded,
                    failed: cs.failed,
                    running: cs.total - cs.succeeded - cs.failed,
                  };
                },
              );
            },
          });
        },

        // TODO: check what `component/check_qualifications` endpoint is doing
        async FETCH_COMPONENT_QUALIFICATIONS(componentId: ComponentId) {
          return new ApiRequest<Qualification[]>({
            url: "component/list_qualifications",
            keyRequestStatusBy: componentId,
            params: {
              componentId,
              visibility_change_set_pk: changeSetId,
              workspaceId,
            },
            onSuccess: (response) => {
              this.qualificationsByComponentId[componentId] = response;
            },
          });
        },
      },
      onActivated() {
        if (!changeSetId) return;

        this.FETCH_QUALIFICATIONS_SUMMARY();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "CheckedQualifications",
            callback: () => {
              this.checkedQualificationsAt = new Date();
              this.FETCH_QUALIFICATIONS_SUMMARY();
            },
          },
          {
            // don't think this should be needed, but not getting other event when new component is added
            eventType: "ChangeSetWritten",
            callback: () => {
              this.FETCH_QUALIFICATIONS_SUMMARY();
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
