import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ComponentId } from "@/store/components.store";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { Resource, ResourceHealth } from "@/api/sdf/dal/resource";
import { useRealtimeStore } from "../realtime/realtime.store";
import { useAuthStore } from "../auth.store";

export type FixStatus = "success" | "failure" | "running" | "unstarted";
export type RecommendationKind = "create" | "other";

export type ConfirmationId = string;
export type ConfirmationResolverId = string;
export type Confirmation = {
  id: ConfirmationId;
  title: string;
  description?: string;
  componentId: ComponentId;
  output?: string[];
  status: "running" | "success" | "failure";
};
export type Recommendation = {
  id: ConfirmationId;
  name: string;
  componentName: string;
  componentId: ComponentId;
  schemaName: string;
  recommendation: string;
  recommendationKind: RecommendationKind;
  status: FixStatus;
  provider?: string;
  output?: string; // TODO(victor): output possibly comes from another endpoint, and should be linked at runtime. This is good for now.
  startedAt?: Date;
  finishedAt?: Date;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
// A potential temporary fix: we decide to convert the "string" from the database row into
// a "date" object within the sdf route(s).
export type FixId = string;
export type Fix = {
  id: FixId;
  status: FixStatus;
  action: string;
  schemaName: string;
  componentName: string;
  componentId: ComponentId;
  resolverId: ConfirmationResolverId;
  provider?: string;
  resource: Resource;
  startedAt: string;
  finishedAt: string;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
export type FixBatchId = string;
export type FixBatch = {
  id: FixBatchId;
  status: FixStatus;
  author: string;
  fixes: Fix[];
  startedAt: string;
  finishedAt: string;
};

export interface ConfirmationStats {
  failure: number;
  success: number;
  running: number;
  total: number;
}

export const useFixesStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspaceId;

  return addStoreHooks(
    defineStore(`w${workspaceId || "NONE"}/fixes`, {
      state: () => ({
        confirmations: [] as Array<Confirmation>,
        recommendations: [] as Array<Recommendation>,
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
        confirmationStats(): ConfirmationStats {
          const obj = { failure: 0, success: 0, running: 0, total: 0 };
          for (const confirmation of this.confirmations) {
            obj[confirmation.status]++;
            obj.total++;
          }
          return obj;
        },
        workspaceStatus(): "running" | "failure" | "success" {
          for (const confirmation of this.confirmations) {
            if (confirmation.status === "running") return "running";
            if (confirmation.status === "failure") return "failure";
          }
          return "success";
        },
        statusByComponentId(): Record<
          ComponentId,
          "success" | "failure" | "running"
        > {
          const map: Record<ComponentId, "success" | "failure" | "running"> =
            {};
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
        allRecommendations(): Recommendation[] {
          return this.recommendations;
        },
        recommendationsByComponentId(): Record<ComponentId, Recommendation> {
          return _.keyBy(this.allRecommendations, (r) => r.componentId);
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
          return this.allRecommendations.filter(
            (recommendation) => recommendation.status === "unstarted",
          );
        },
      },
      actions: {
        async LOAD_CONFIRMATIONS() {
          this.populatingFixes = true;

          return new ApiRequest<Array<Confirmation>>({
            url: "/fix/confirmations",
            params: { visibility_change_set_pk: -1 },
            onSuccess: (response) => {
              this.confirmations = response;
              this.populatingFixes =
                response.length === 0 ||
                response.some((c) => c.status === "running");
            },
          });
        },
        async LOAD_RECOMMENDATIONS() {
          return new ApiRequest<Array<Recommendation>>({
            url: "/fix/recommendations",
            params: { visibility_change_set_pk: -1 },
            onSuccess: (response) => {
              this.recommendations = response;
            },
          });
        },
        async LOAD_FIX_BATCHES() {
          return new ApiRequest<Array<FixBatch>>({
            url: "/fix/list",
            params: { visibility_change_set_pk: -1 },
            onSuccess: (response) => {
              this.fixBatches = response;
            },
          });
        },
        async EXECUTE_FIXES_FROM_RECOMMENDATIONS(
          recommendations: Array<Recommendation>,
        ) {
          const authStore = useAuthStore();

          return new ApiRequest({
            method: "post",
            params: {
              list: recommendations.map((fix) => ({
                id: fix.id,
                componentId: fix.componentId,
                actionName: fix.recommendation,
              })),
              visibility_change_set_pk: -1,
            },
            url: "/fix/run",
            onSuccess: (response) => {
              this.recommendations = this.recommendations.map((r) => {
                if (recommendations.find((rec) => rec.id === r.id)) {
                  r.status = "running";
                }
                return r;
              });

              this.runningFixBatch = response.id;
              this.fixBatches = this.fixBatches.filter(
                (b) => b.id !== response.id,
              );
              this.fixBatches.push({
                id: response.id,
                status: "running",
                fixes: recommendations.map((r) => {
                  return {
                    id: r.id,
                    resolverId: r.id,
                    status: "running" as FixStatus,
                    action: r.recommendation,
                    schemaName: r.schemaName,
                    componentName: r.componentName,
                    componentId: r.componentId,
                    resource: {
                      status: ResourceHealth.Ok as ResourceHealth,
                      data: null,
                      message: null,
                      logs: [],
                    },
                    startedAt: `${new Date()}`,
                    finishedAt: `${new Date()}`,
                  };
                }),
                author: authStore.user?.email ?? "...",
                startedAt: `${new Date()}`,
                finishedAt: `${new Date()}`,
              });
            },
          });
        },
        updateRecommendation(recommendation: Recommendation) {
          const index = this.recommendations.findIndex(
            (r) => r.id === recommendation.id,
          );
          if (index === -1) {
            this.recommendations.push(recommendation);
          } else {
            this.recommendations[index] = recommendation;
          }
        },
      },
      async onActivated() {
        this.LOAD_RECOMMENDATIONS();
        this.LOAD_CONFIRMATIONS();
        this.LOAD_FIX_BATCHES();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}/head`, [
          {
            eventType: "ConfirmationStatusUpdate",
            callback: (_update) => {
              this.runningFixBatch = undefined;

              // we could be smarter with this
              this.LOAD_CONFIRMATIONS();
              this.LOAD_RECOMMENDATIONS();
              this.LOAD_FIX_BATCHES();
            },
          },
          {
            eventType: "FixReturn",
            callback: (update) => {
              const batch = this.fixBatches.find(
                (b) => b.id === update.batchId,
              );
              if (!batch) {
                this.LOAD_RECOMMENDATIONS();
                this.LOAD_FIX_BATCHES();
                return;
              }
              const fix = batch.fixes.find(
                (f) =>
                  f.resolverId === update.confirmationResolverId &&
                  f.action === update.action,
              );
              if (!fix) {
                this.LOAD_RECOMMENDATIONS();
                this.LOAD_FIX_BATCHES();
                return;
              }
              this.recommendations = this.recommendations.map((r) => {
                if (r.id === fix.id) {
                  r.status = update.status;
                }
                return r;
              });
              if (update.status !== fix.status) {
                fix.status = update.status;
                fix.action = update.action;
              }
            },
          },
          {
            eventType: "FixBatchReturn",
            callback: (update) => {
              const batch = this.fixBatches.find((b) => b.id === update.id);
              if (!batch) {
                this.LOAD_RECOMMENDATIONS();
                this.LOAD_FIX_BATCHES();
                return;
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
