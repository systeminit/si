import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { nilId } from "@/utils/nilId";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ComponentId } from "@/store/components.store";
import { Resource } from "@/api/sdf/dal/resource";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { AttributeValueId } from "./status.store";
import { trackEvent } from "../utils/tracking";

export type ActionPrototypeId = string;

export type FixStatus =
  | "success"
  | "failure"
  | "running"
  | "error"
  | "unstarted";

export enum ActionKind {
  Create = "create",
  Delete = "delete",
  Other = "other",
  Refresh = "refresh",
}

// TODO(nick,paulo,paul,wendy): get rid of never started.
export type ConfirmationStatus =
  | "success"
  | "failure"
  | "running"
  | "neverStarted";

export type Confirmation = {
  attributeValueId: AttributeValueId;
  title: string;
  description?: string;

  schemaId: string;
  schemaVariantId: string;
  componentId: ComponentId;

  schemaName: string;
  componentName: string;

  output?: string[];
  status: ConfirmationStatus;
  provider?: string;
};

export type Recommendation = {
  confirmationAttributeValueId: AttributeValueId;
  componentId: ComponentId;
  componentName: string;
  name: string;
  actionPrototypeId: ActionPrototypeId;
  provider: string;
  actionKind: ActionKind;
  hasRunningFix: boolean;
  lastFix?: Fix;
};

export type FixId = string;
export type Fix = {
  id: FixId;
  status: FixStatus;
  actionKind: string;
  schemaName: string;
  componentName: string;
  componentId: ComponentId;
  attributeValueId: AttributeValueId;
  provider?: string;
  resource?: Resource | null;
  startedAt?: string;
  finishedAt?: string;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
export type FixBatchId = string;
export type FixBatch = {
  id: FixBatchId;
  status?: FixStatus;
  author: string;
  fixes: Fix[];
  startedAt?: string;
  finishedAt?: string;
};

export interface ConfirmationStats {
  failure: number;
  success: number;
  running: number;
  neverStarted: number;
  total: number;
}

export const useFixesStore = () => {
  const changeSetStore = useChangeSetsStore();
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;
  const changeSetId = useChangeSetsStore().selectedChangeSetId;
  const name = `w${workspacePk || "NONE"}/cs${changeSetId || "NONE"}/fixes`;

  return addStoreHooks(
    defineStore(name, {
      state: () => ({
        confirmations: [] as Array<Confirmation>,
        recommendations: [] as Array<Recommendation>,
        fixBatches: [] as Array<FixBatch>,
        runningFixBatch: undefined as FixBatchId | undefined,
        populatingFixes: false,
        recommendationsSelection: {} as Record<
          string,
          { recommendation: Recommendation; selected: boolean }
        >,
      }),
      getters: {
        fixesAreInProgress: (state) => !!state.runningFixBatch,
        enabledRecommendations(): Recommendation[] {
          return _.values(this.recommendationsSelection)
            .filter(({ selected }) => selected)
            .map(({ recommendation }) => recommendation);
        },
        allComponents(): ComponentId[] {
          return _.uniq(this.confirmations.map((c) => c.componentId));
        },
        confirmationsByComponentId(): Record<ComponentId, Confirmation[]> {
          const obj: Record<ComponentId, Confirmation[]> = {};
          for (const confirmation of this.confirmations) {
            obj[confirmation.componentId] ||= [];
            obj[confirmation.componentId]!.push(confirmation); // eslint-disable-line @typescript-eslint/no-non-null-assertion
          }
          return obj;
        },
        confirmationStatusByComponentId(): Record<
          ComponentId,
          ConfirmationStatus | undefined
        > {
          const obj: Record<ComponentId, ConfirmationStatus | undefined> = {};
          for (const compId in this.confirmationsByComponentId) {
            const confirmations = this.confirmationsByComponentId[compId];
            if (!obj[compId]) {
              obj[compId] = _.reduce(
                confirmations,
                (collector: ConfirmationStatus | undefined, confirmation) => {
                  if (confirmation === undefined) return;
                  if (collector === "failure") return collector;
                  else if (collector === "running")
                    return confirmation.status === "failure"
                      ? "failure"
                      : collector;
                  else if (collector === "success")
                    return confirmation.status ?? "success";
                  else return confirmation.status;
                },
                undefined,
              );
            }
          }
          return obj;
        },
        confirmationStats(): ConfirmationStats {
          const obj = {
            failure: 0,
            success: 0,
            running: 0,
            neverStarted: 0,
            total: 0,
          };
          for (const confirmation of this.confirmations) {
            obj[confirmation.status]++;
            obj.total++;
          }
          return obj;
        },
        workspaceStatus(): ConfirmationStatus {
          for (const confirmation of this.confirmations) {
            if (confirmation.status === "running") return "running";
            if (confirmation.status === "failure") return "failure";
          }
          return "success";
        },
        statusByComponentId(): Record<ComponentId, ConfirmationStatus> {
          const map = _.pickBy(
            this.confirmationStatusByComponentId,
            (c) => !!c,
          ) as Record<ComponentId, ConfirmationStatus>;
          for (const fixes of this.fixesOnRunningBatch) {
            switch (fixes.status) {
              case "success":
                if (!map[fixes.componentId]) map[fixes.componentId] = "success";
                break;
              case "failure":
                if (map[fixes.componentId] !== "running")
                  map[fixes.componentId] = "failure";
                break;
              case "running":
                map[fixes.componentId] = "running";
                break;
              default:
                break;
            }
          }
          return map;
        },
        finishedConfirmations(): Confirmation[] {
          return this.confirmations.filter((c) => c.status !== "running");
        },
        allFinishedFixBatches(): FixBatch[] {
          return this.fixBatches.filter(
            (f) => f.status !== "running" && f.status !== "unstarted",
          );
        },
        fixesOnBatch() {
          return (fixBatchId: FixBatchId) => {
            for (const batch of this.fixBatches) {
              if (batch.id === fixBatchId) {
                return batch.fixes;
              }
            }
            return [];
          };
        },
        completedFixesOnRunningBatch(): Fix[] {
          return _.filter(
            this.fixesOnRunningBatch,
            (fix) => !["running", "unstarted"].includes(fix.status),
          );
        },
        fixesOnRunningBatch(): Fix[] {
          if (!this.runningFixBatch) return [];
          return this.fixesOnBatch(this.runningFixBatch);
        },
        newRecommendations(): Recommendation[] {
          return this.recommendations.filter(
            (recommendation) => recommendation.lastFix === undefined,
          );
        },
      },
      actions: {
        async LOAD_CONFIRMATIONS() {
          this.populatingFixes = true;

          return new ApiRequest<{
            confirmations: Array<Confirmation>;
            recommendations: Array<Recommendation>;
          }>({
            url: "/fix/confirmations",
            params: {
              visibility_change_set_pk:
                changeSetStore.selectedChangeSetId ?? nilId(),
            },
            onSuccess: ({ confirmations, recommendations }) => {
              this.confirmations = confirmations;
              this.recommendations = recommendations;
              this.recommendationsSelection = {};
              for (const recommendation of this.recommendations) {
                const key = `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`;
                this.recommendationsSelection[key] = {
                  recommendation,
                  selected: true,
                };
              }

              this.populatingFixes =
                confirmations.some((c) => c.status === "neverStarted") ||
                confirmations.some((c) => c.status === "running");
            },
          });
        },
        async LOAD_FIX_BATCHES() {
          return new ApiRequest<Array<FixBatch>>({
            url: "/fix/list",
            params: {
              visibility_change_set_pk: nilId(),
            },
            onSuccess: (response) => {
              this.fixBatches = response;
              this.runningFixBatch = response.find(
                (batch) => !["success", "failure"].includes(batch.status ?? ""),
              )?.id;
            },
          });
        },
        async EXECUTE_FIXES_FROM_RECOMMENDATIONS(
          recommendations: Array<Recommendation>,
        ) {
          return new ApiRequest({
            method: "post",
            params: {
              list: recommendations.map((r) => ({
                attributeValueId: r.confirmationAttributeValueId,
                componentId: r.componentId,
                actionPrototypeId: r.actionPrototypeId,
              })),
              visibility_change_set_pk: nilId(),
            },
            url: "/fix/run",
            onSuccess: (response) => {
              this.LOAD_CONFIRMATIONS();
              this.LOAD_FIX_BATCHES();
            },
          });
        },
      },
      async onActivated() {
        this.LOAD_CONFIRMATIONS();
        this.LOAD_FIX_BATCHES();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(
          this.$id,
          `workspace/${workspacePk}/${
            changeSetStore.selectedChangeSetId ?? "head"
          }`,
          [
            {
              eventType: "ChangeSetWritten",
              callback: (writtenChangeSetId) => {
                if (writtenChangeSetId !== changeSetStore.selectedChangeSetId)
                  return;
                this.LOAD_CONFIRMATIONS();
                this.LOAD_FIX_BATCHES();
              },
            },
            {
              eventType: "ConfirmationsUpdated",
              callback: (_update) => {
                this.LOAD_CONFIRMATIONS();
                this.LOAD_FIX_BATCHES();
              },
            },
            {
              eventType: "FixReturn",
              callback: (update) => {
                trackEvent("fix_return", {
                  fix_action: update.action,
                  fix_status: update.status,
                  fix_id: update.id,
                  fix_batch_id: update.batchId,
                });

                this.LOAD_CONFIRMATIONS();
                this.LOAD_FIX_BATCHES();
              },
            },
            {
              eventType: "FixBatchReturn",
              callback: (update) => {
                this.runningFixBatch = undefined;
                trackEvent("fix_batch_return", {
                  batch_status: update.status,
                  batch_id: update.id,
                });

                this.LOAD_CONFIRMATIONS();
                this.LOAD_FIX_BATCHES();
              },
            },
          ],
        );

        return () => {
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
