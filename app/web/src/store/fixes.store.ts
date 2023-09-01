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

export const useFixesStore = () => {
  const changeSetStore = useChangeSetsStore();
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;
  const changeSetId = useChangeSetsStore().selectedChangeSetId;
  const name = `w${workspacePk || "NONE"}/cs${changeSetId || "NONE"}/fixes`;

  return addStoreHooks(
    defineStore(name, {
      state: () => ({
        fixBatches: [] as Array<FixBatch>,
        runningFixBatch: undefined as FixBatchId | undefined,
        populatingFixes: false,
      }),
      getters: {
        fixesAreInProgress: (state) => !!state.runningFixBatch,
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
      },
      actions: {
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
      },
      async onActivated() {
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
