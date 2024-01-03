import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { Qualification } from "@/api/sdf/dal/qualification";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { ComponentId, useComponentsStore } from "./components.store";

export type QualificationStatus = "success" | "failure" | "running" | "warning";

// TODO: align these key names with the status (ex: succeeded -> success)
type QualificationStats = {
  total: number;
  succeeded: number;
  warned: number;
  failed: number;
  running: number;
};

export const useQualificationsStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/qualifications`,
      {
        state: () => ({
          // stats per component - this is the raw data
          // we may change this to store qualification ids and individual statuses to make realtime updates easier
          // NOTE(victor) Use the qualificationStatsByComponentId getter, it has validation data
          qualificationStatsByComponentIdRaw: {} as Record<
            ComponentId,
            QualificationStats
          >,

          // NOTE(victor) Use the qualificationsByComponentId getter, it has validation data
          qualificationsByComponentIdRaw: {} as Record<
            ComponentId,
            Qualification[]
          >,

          checkedQualificationsAt: null as Date | null,
        }),
        getters: {
          // NOTE(victor) the following two getters only exist because Joi validations
          // run on the frontend. If all qualification data goes back
          // to coming from the API we can delete both *Raw entries from state
          qualificationStatsByComponentId: (state) =>
            _.mapValues(
              state.qualificationStatsByComponentIdRaw,
              (cs, componentId) => {
                const { result: validationResult } =
                  useComponentAttributesStore(componentId).schemaValidation;

                let total = cs.total;
                let succeeded = cs.succeeded;
                let failed = cs.failed;

                if (validationResult) {
                  total += 1;
                  if (validationResult.status === "success") succeeded += 1;
                  else failed += 1;
                }

                return {
                  ...cs,
                  total,
                  succeeded,
                  failed,
                };
              },
            ),
          qualificationsByComponentId: (state) =>
            _.mapValues(
              state.qualificationsByComponentIdRaw,
              (qualifications, componentId) => {
                const qualificationsAndValidation = [
                  ...qualifications,
                  useComponentAttributesStore(componentId).schemaValidation,
                ];

                // TODO: maybe we want to sort these in the backend?
                return _.orderBy(
                  qualificationsAndValidation,
                  (response) =>
                    ({
                      failure: 1,
                      warning: 2,
                      unknown: 3,
                      success: 4,
                    }[response.result?.status || "unknown"]),
                );
              },
            ),

          // single status per component
          qualificationStatusByComponentId(): Record<
            ComponentId,
            QualificationStatus
          > {
            return _.mapValues(this.qualificationStatsByComponentId, (cs) => {
              if (cs.running) return "running";
              if (cs.failed > 0) return "failure";
              if (cs.warned > 0) return "warning";
              return "success";
            });
          },
          qualificationStatusWithRollupsByComponentId(): Record<
            ComponentId,
            QualificationStatus | undefined
          > {
            const componentsStore = useComponentsStore();

            return _.mapValues(
              componentsStore.componentsById,
              (component, id) => {
                if (!component.isGroup)
                  return this.qualificationStatusByComponentId[id];

                const deepChildIds =
                  componentsStore.deepChildIdsByComponentId[id];
                const deepChildStatuses = _.map(
                  deepChildIds,
                  (childId) => this.qualificationStatusByComponentId[childId],
                );
                if (_.some(deepChildStatuses, (s) => s === "running"))
                  return "running";
                if (_.some(deepChildStatuses, (s) => s === "failure"))
                  return "failure";
                if (_.some(deepChildStatuses, (s) => s === "warning"))
                  return "warning";
                return "success";
              },
            );
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
          async FETCH_QUALIFICATIONS_SUMMARY() {
            return new ApiRequest<{
              total: number;
              succeeded: number;
              warned: number;
              failed: number;
              components: {
                componentId: string;
                componentName: string;
                total: number;
                warned: number;
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
                const byComponentId = _.keyBy(
                  response.components,
                  "componentId",
                );

                this.qualificationStatsByComponentIdRaw = _.mapValues(
                  byComponentId,
                  ({ total, succeeded, warned, failed }) => ({
                    // transform the data slightly to add "running" so we can avoid recalculating again elsewhere
                    total,
                    succeeded,
                    warned,
                    failed,
                    running: total - succeeded - failed - warned,
                  }),
                );
              },
            });
          },

          async FETCH_COMPONENT_QUALIFICATIONS(componentId: ComponentId) {
            // Do not fetch qualifications for a deleted component
            // const componentsStore = useComponentsStore();
            // const component = componentsStore.componentsById[componentId];
            // if (component?.changeStatus === "deleted") return;

            return new ApiRequest<Qualification[]>({
              url: "component/list_qualifications",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                visibility_change_set_pk: changeSetId,
              },
              onSuccess: (response) => {
                this.qualificationsByComponentIdRaw[componentId] = response;
              },
            });
          },
        },
        onActivated() {
          if (!changeSetId) return;

          this.FETCH_QUALIFICATIONS_SUMMARY();

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            // Doesnt seem to actually do anything
            // {
            //   eventType: "CheckedQualifications",
            //   callback: ({ componentId }) => {
            //     this.FETCH_COMPONENT_QUALIFICATIONS(componentId);
            //   },
            // },
            // {
            //   eventType: "ComponentCreated",
            //   callback: () => {
            //     this.FETCH_QUALIFICATIONS_SUMMARY();
            //   },
            // },
            {
              // TODO(nick,theo,fletcher,wendy): replace this someday.
              eventType: "ChangeSetWritten",
              debounce: true,
              callback: () => {
                this.FETCH_QUALIFICATIONS_SUMMARY();
              },
            },
          ]);

          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
