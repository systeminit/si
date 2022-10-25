import { defineStore } from "pinia";
import _ from "lodash";
import { watch } from "vue";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import promiseDelay from "@/utils/promise_delay";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { User } from "@/api/sdf/dal/user";
import { Visibility } from "@/api/sdf/dal/visibility";
import { useAuthStore } from "@/store/auth.store";
import { ResourceStatus } from "@/api/sdf/dal/resource";
import { useResourcesStore } from "../resources.store";
import { useRealtimeStore } from "../realtime/realtime.store";

export type FixStatus = "success" | "failure" | "running" | "unstarted";

export type FixId = number;
export type Confirmation = {
  id: FixId;
  status: "running" | "finished";
}
export type Fix = {
  id: FixId;
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

export type FixBatchId = number;
export type FixBatch = {
  id: FixBatchId;
  author: User;
  timestamp: Date;
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
        fixes: [] as Array<Fix>,
        fixBatchIdsByFixId: {} as Record<FixId, FixBatchId>,
        fixBatchesById: {} as Record<FixBatchId, FixBatch>,
        runningFixBatch: undefined as FixBatchId | undefined,
        populatingFixes: false,
      }),
      getters: {
        finishedConfirmations(): Confirmation[] {
          return this.confirmations.filter((c) => c.status !== "running");
        },
        allFixes(): Fix[] {
          return this.fixes;
        },
        fixesByComponentId(): Record<ComponentId, Fix> {
          return _.keyBy(this.allFixes, (f) => f.componentId);
        },
        allFixBatches(): FixBatch[] {
          return _.values(this.fixBatchesById);
        },
        fixesOnBatch() {
          return (fixBatchId: FixBatchId) => {
            const fixes = [];

            for (const fixId in this.fixBatchIdsByFixId) {
              if (this.fixBatchIdsByFixId[fixId] === fixBatchId) {
                const fix = this.fixes.find((fix) => String(fix.id) === fixId);
                if (fix) fixes.push(fix);
              }
            }

            return fixes;
          };
        },
        fixesOnRunningBatch(): Fix[] {
          if (!this.runningFixBatch) return [];

          return this.fixesOnBatch(this.runningFixBatch);
        },
        completedFixesOnRunningBatch(): Fix[] {
          return _.filter(
            this.fixesOnRunningBatch,
            (fix) => fix.status !== "unstarted",
          );
        },
        unstartedFixes(): Fix[] {
          return _.filter(this.allFixes, (fix) => fix.status === "unstarted");
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
              this.populatingFixes = response.length === 0 || response.some((c) => c.status === "running");
            },
          });
        },
        async LOAD_FIXES() {
          this.runningFixBatch = undefined; // TODO: backend should tell us that
          return new ApiRequest<Array<Fix>>({
            url: "/fix/list",
            params: { visibility_change_set_pk: -1 },
            onSuccess: (response) => {
              this.fixes = response;
            },
          });
        },
        async EXECUTE_FIXES(fixes: Array<Fix>) {
          // TODO: have an actual FixBatch related to this run in the backend
          this.runningFixBatch = -1;
          this.fixBatchIdsByFixId = {};
          for (const fix of fixes) {
            this.fixBatchIdsByFixId[fix.id] = -1;
          }

          return new ApiRequest({
            method: "post",
            params: {
              list: fixes.map((fix) => ({
                id: fix.id,
                componentId: fix.componentId,
                actionName: fix.recommendation,
              })),
              visibility_change_set_pk: -1,
            },
            url: "/fix/run",
            onSuccess: (response) => {},
          });
        },
        updateFix(fix: Fix) {
          const index = this.fixes.findIndex((f) => f.id === fix.id);
          if (index === -1) {
            this.fixes.push(fix);
          } else {
            this.fixes[index] = fix;
          }
        },
      },
      async onActivated() {
        const resourcesStore = useResourcesStore();
        // What purpose does this serve?
        await resourcesStore.generateMockResources();
        this.LOAD_FIXES();
        this.LOAD_CONFIRMATIONS();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}/head`, [
          {
            eventType: "ConfirmationStatusUpdate",
            callback: (update) => {
              // we could be smarter with this
              this.LOAD_CONFIRMATIONS();
              this.LOAD_FIXES();
            },
          },
          {
            eventType: "FixReturn",
            callback: (update) => {
              const fix = this.fixes.find((f) => f.id === update.confirmationResolverId && f.recommendation === update.action);
              if (!fix) {
                this.LOAD_FIXES();
                return;
              }
              switch (update.runnerState.status) {
                case "success":
                  fix.status = "success";
                  break;
                case "failure":
                  fix.status = "failure";
                  break;
                case "running":
                  fix.status = "running";
                  break;
                case "created":
                  fix.status = "unstarted";
                  break;
                default:
                  break;
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
