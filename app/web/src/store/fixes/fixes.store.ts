import { defineStore } from "pinia";
import _ from "lodash";
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
import { useChangeSetsStore } from "../change_sets.store";

export type FixStatus = "success" | "failure" | "running" | "unstarted";

export type FixId = number;
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

let batchIdCounter = 1;

export const useFixesStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspaceId;

  return addStoreHooks(
    defineStore(`w${workspaceId || "NONE"}/fixes`, {
      state: () => ({
        fixes: [] as Array<Fix>,
        fixBatchIdsByFixId: {} as Record<FixId, FixBatchId>,
        fixBatchesById: {} as Record<FixBatchId, FixBatch>,
        processedFixComponents: 0,
        runningFixBatch: undefined as FixBatchId | undefined,
        populatingFixes: false,
      }),
      getters: {
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
            (fix) => fix.status === "success",
          );
        },
        unstartedFixes(): Fix[] {
          return _.filter(this.allFixes, (fix) => fix.status === "unstarted");
        },
      },
      actions: {
        async LOAD_FIXES() {
          const changeSetStore = useChangeSetsStore();
          const selectedChangeSetId = changeSetStore.selectedChangeSetId;
          const visibility: Visibility = {
            visibility_change_set_pk: selectedChangeSetId ?? -1,
          };
          return new ApiRequest<Array<Fix>>({
            url: "/fix/list",
            params: { ...visibility },
            onSuccess: (response) => {
              this.fixes = response;
            },
          });
        },
        async EXECUTE_FIXES(fixes: Array<Fix>) {
          const changeSetStore = useChangeSetsStore();
          const selectedChangeSetId = changeSetStore.selectedChangeSetId;
          const visibility: Visibility = {
            visibility_change_set_pk: selectedChangeSetId ?? -1,
          };

          return new ApiRequest({
            method: "post",
            params: {
              list: fixes.map((fix) => ({
                id: fix.id,
                componentId: fix.componentId,
                actionName: fix.recommendation,
              })),
              ...visibility,
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
        async executeMockFixes(fixes: Array<Fix>) {
          const authStore = useAuthStore();
          const componentsStore = useComponentsStore(-1);
          const resourcesStore = useResourcesStore();

          const fixBatch = <FixBatch>{
            id: batchIdCounter++,
            author: authStore.user,
            timestamp: new Date(),
          };

          this.fixBatchesById[fixBatch.id] = fixBatch;

          this.runningFixBatch = fixBatch.id;

          for (const fix of fixes) {
            this.fixBatchIdsByFixId[fix.id] = fixBatch.id;
          }

          for (const fix of fixes) {
            await promiseDelay(200);

            this.updateFix({
              ...fix,
              startedAt: new Date(),
              status: "running",
            });

            // not shown anywhere at the moment
            resourcesStore.resourcesByComponentId[fix.componentId].status =
              ResourceStatus.InProgress;

            componentsStore.increaseActivityCounterOnComponent(fix.componentId);

            const component = componentsStore.componentsById[fix.componentId];

            await promiseDelay(
              SCHEMA_MOCK_METADATA[component.schemaName]?.fixDelay || 2000,
            );

            this.updateFix({
              ...fix,
              finishedAt: new Date(),
              status: "success",
            });
            componentsStore.decreaseActivityCounterOnComponent(fix.componentId);

            const confirmation =
              resourcesStore.confirmationsByComponentId[fix.componentId][0];
            confirmation.status = "running";
            await promiseDelay(1000);
            confirmation.status = "success";
            confirmation.description = "This resource exists!";

            resourcesStore.resourcesByComponentId[fix.componentId].status =
              ResourceStatus.Created;

            if (["EC2 Instance"].includes(component.schemaName))
              resourcesStore.resourcesByComponentId[fix.componentId].link =
                "https://www.youtube.com/watch?v=fzcSJ1setd0"; // TODO Replace with actual whiskers r we link/
          }
          await promiseDelay(1600); // delay time for UI to update
          this.runningFixBatch = undefined;
        },
      },
      async onActivated() {
        const resourcesStore = useResourcesStore();
        await resourcesStore.generateMockResources();
        await this.LOAD_FIXES();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}/head`, [
          {
            eventType: "ChangeSetWritten",
            callback: (writtenChangeSetId) => {
              if (writtenChangeSetId === -1) this.LOAD_FIXES();
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
