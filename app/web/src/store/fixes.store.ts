import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ComponentId } from "@/store/components.store";
import { Resource } from "@/api/sdf/dal/resource";
import { useRealtimeStore } from "./realtime/realtime.store";
import { AttributeValueId } from "./status.store";

function nilId(): string {
  return "00000000000000000000000000";
}

export type FixStatus = "success" | "failure" | "running" | "unstarted";
export type RecommendationStatus =
  | "success"
  | "failure"
  | "running"
  | "unstarted";
export type RecommendationIsRunnable = "yes" | "no" | "running";
export type ActionKind = "create" | "other" | "destroy";

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
  recommendations: Recommendation[];
};

export type Recommendation = {
  confirmationAttributeValueId: AttributeValueId;
  componentId: ComponentId;
  componentName: string;
  name: string;
  recommendedAction: string;
  provider: string;
  actionKind: ActionKind;
  status: RecommendationStatus; // TODO(Wendy) - this should be replaced with a reference to the lastFixRun
  lastFix?: Fix; // TODO(nick,wendy): delete status if we don't need it
  isRunnable: RecommendationIsRunnable;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
// A potential temporary fix: we decide to convert the "string" from the database row into

export type Fix = {
  status: FixStatus;
  action: string;
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

// TODO(nick,paulo,paul,wendy): get rid of never started.
export type ConfirmationStatus =
  | "success"
  | "failure"
  | "running"
  | "neverStarted";
export const useFixesStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;

  return addStoreHooks(
    defineStore(`w${workspacePk || "NONE"}/fixes`, {
      state: () => ({
        confirmations: [] as Array<Confirmation>,
        fixBatches: [] as Array<FixBatch>,
        runningFixBatch: undefined as FixBatchId | undefined,
        populatingFixes: false,
      }),
      getters: {
        allComponents(): ComponentId[] {
          return _.uniq(this.confirmations.map((c) => c.componentId));
        },
        confirmationsByComponentId(): Record<ComponentId, Confirmation[]> {
          const obj: Record<ComponentId, Confirmation[]> = {};
          for (const confirmation of this.confirmations) {
            if (!obj[confirmation.componentId])
              obj[confirmation.componentId] = [];
            obj[confirmation.componentId].push(confirmation);
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
        unstartedRecommendations(): Recommendation[] {
          return this.confirmations.flatMap((c) =>
            c.recommendations.filter(
              (recommendation) => recommendation.status === "unstarted",
            ),
          );
        },
      },
      actions: {
        async LOAD_CONFIRMATIONS() {
          this.populatingFixes = true;

          return new ApiRequest<Array<Confirmation>>({
            url: "/fix/confirmations",
            params: { visibility_change_set_pk: nilId() },
            onSuccess: (response) => {
              this.confirmations = response;
              this.populatingFixes =
                response.some((c) => c.status === "neverStarted") ||
                response.some((c) => c.status === "running");
            },
          });
        },
        async LOAD_FIX_BATCHES() {
          return new ApiRequest<Array<FixBatch>>({
            url: "/fix/list",
            params: { visibility_change_set_pk: nilId() },
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
                actionName: r.recommendedAction,
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
        realtimeStore.subscribe(this.$id, `workspace/${workspacePk}/head`, [
          {
            eventType: "ConfirmationsUpdated",
            callback: (_update) => {
              this.LOAD_CONFIRMATIONS();
              this.LOAD_FIX_BATCHES();
            },
          },
          {
            eventType: "FixReturn",
            callback: (_update) => {
              this.LOAD_CONFIRMATIONS();
              this.LOAD_FIX_BATCHES();
            },
          },
          {
            eventType: "FixBatchReturn",
            callback: (_update) => {
              this.LOAD_CONFIRMATIONS();
              this.LOAD_FIX_BATCHES();
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
