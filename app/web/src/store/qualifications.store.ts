import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { Qualification } from "@/api/sdf/dal/qualification";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { DiagramStatusIcon } from "@/components/ModelingDiagram/diagram_types";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import handleStoreError from "./errors";

export type QualificationStatus = "success" | "failure" | "running" | "warning";

// TODO: align these key names with the status (ex: succeeded -> success)
type QualificationStats = {
  total: number;
  succeeded: number;
  warned: number;
  failed: number;
  running: number;
};

export const qualificationStatusToIconMap: Record<QualificationStatus | "notexists", DiagramStatusIcon> = {
  success: { icon: "check-hex-outline", tone: "success" },
  warning: { icon: "check-hex-outline", tone: "warning" },
  failure: { icon: "x-hex-outline", tone: "error" },
  running: { icon: "loader", tone: "info" },
  notexists: { icon: "none" },
};

export const statusIconsForComponent = (qualificationStatus?: QualificationStatus, hasResource?: boolean) => {
  const statusIcons: DiagramStatusIcon[] = _.compact([
    {
      ...qualificationStatusToIconMap[qualificationStatus ?? "notexists"],
      tabSlug: "qualifications",
    },
    hasResource ? { icon: "check-hex", tone: "success", tabSlug: "resource" } : { icon: "none" },
  ]);

  return statusIcons;
};

export const useQualificationsStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;

  const realtimeStore = useRealtimeStore();
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/qualifications`, {
      state: () => ({
        qualificationStatsByComponentId: {} as Record<ComponentId, QualificationStats>,

        // NOTE(victor) Use the qualificationsByComponentId getter, it has validation data
        qualificationsByComponentId: {} as Record<ComponentId, Qualification[]>,

        checkedQualificationsAt: null as Date | null,
      }),
      getters: {
        // single status per component
        qualificationStatusByComponentId(): Record<ComponentId, QualificationStatus> {
          return _.mapValues(this.qualificationStatsByComponentId, (cs) => {
            if (cs.running) return "running";
            if (cs.failed > 0) return "failure";
            if (cs.warned > 0) return "warning";
            return "success";
          });
        },
        qualificationStatusForComponentId:
          (state) =>
          (componentId: ComponentId): QualificationStatus => {
            const stats = state.qualificationStatsByComponentId[componentId];
            if (stats) {
              if (stats.running) {
                return "running";
              }
              if (stats.failed > 0) {
                return "failure";
              }
              if (stats.warned > 0) {
                return "warning";
              }
            }

            return "success";
          },

        // stats/totals by component
        componentStats(): Record<QualificationStatus | "total", number> {
          const grouped = _.groupBy(this.qualificationStatusByComponentId);
          return {
            failure: grouped.failure?.length || 0,
            success: grouped.success?.length || 0,
            warning: grouped.warning?.length || 0,
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
        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },
      async onActivated() {
        if (!changeSetId) return;

        const actionUnsub = this.$onAction(handleStoreError);

        return () => {
          actionUnsub();
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
