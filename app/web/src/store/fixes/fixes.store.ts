import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ComponentId } from "@/store/components.store";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { useResourcesStore } from "../resources.store";
import { useRealtimeStore } from "../realtime/realtime.store";

export type FixStatus = "success" | "failure" | "running" | "unstarted";

export type ConfirmationId = number;
export type Confirmation = {
  id: ConfirmationId;
  status: "running" | "finished";
};
export type Recommendation = {
  id: ConfirmationId;
  name: string;
  componentName: string;
  componentId: ComponentId;
  recommendation: string;
  status: FixStatus;
  provider?: string;
  output?: string; // TODO(victor): output possibly comes from another endpoint, and should be linked at runtime. This is good for now.
  startedAt?: Date;
  finishedAt?: Date;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
// A potential temporary fix: we decide to convert the "string" from the database row into
// a "date" object within the sdf route(s).
export type FixId = number;
export type Fix = {
  id: FixId;
  status: FixStatus;
  action: string;
  componentName: string;
  componentId: ComponentId;
  provider?: string;
  output?: string;
  startedAt: string;
  finishedAt: string;
};

// TODO(nick): use real user data and real timestamps. This is dependent on the backend.
export type FixBatchId = number;
export type FixBatch = {
  id: FixBatchId;
  status: FixStatus;
  author: string;
  fixes: Fix[];
  startedAt: string;
  finishedAt: string;
};

export const SCHEMA_MOCK_METADATA: Record<
  string,
  { provider: string; fixDelay: number; order: number }
> = {
  AMI: { provider: "AWS", fixDelay: 0, order: 0 },
  "EC2 Instance": { provider: "AWS", fixDelay: 15000, order: 40 },
  Egress: { provider: "AWS", fixDelay: 3000, order: 31 },
  Ingress: { provider: "AWS", fixDelay: 3000, order: 30 },
  "Key Pair": { provider: "AWS", fixDelay: 5000, order: 10 },
  Region: { provider: "AWS", fixDelay: 0, order: 0 },
  "Security Group": { provider: "AWS", fixDelay: 5000, order: 20 },
  Butane: { provider: "CoreOS", fixDelay: 0, order: 0 },
  "Kubernetes Deployment": { provider: "Kubernetes", fixDelay: 1000, order: 0 },
  "Kubernetes Namespace": { provider: "Kubernetes", fixDelay: 500, order: 0 },
  "Docker Image": { provider: "Docker", fixDelay: 0, order: 0 },
  "Docker Hub Credential": { provider: "Docker", fixDelay: 0, order: 0 },
};

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
          return _.values(this.fixBatches);
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
          this.runningFixBatch = undefined; // TODO: backend should tell us that
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
          // TODO: have an actual FixBatch related to this run in the backend
          this.runningFixBatch = -1;
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
            onSuccess: (_response) => {},
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
        const resourcesStore = useResourcesStore();
        // What purpose does this serve?
        await resourcesStore.generateMockResources();
        this.LOAD_RECOMMENDATIONS();
        this.LOAD_CONFIRMATIONS();
        this.LOAD_FIX_BATCHES();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}/head`, [
          {
            eventType: "ConfirmationStatusUpdate",
            callback: (_update) => {
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
              const fix = batch.fixes.find((f) => f.id === update.id);
              if (!fix) {
                this.LOAD_RECOMMENDATIONS();
                this.LOAD_FIX_BATCHES();
                return;
              }
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
              if (update.status !== batch.status) {
                batch.status = update.status;
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
