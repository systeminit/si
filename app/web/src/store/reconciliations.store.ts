import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { nilId } from "@/utils/nilId";
import { ComponentId } from "@/store/components.store";
import { AttributeValueId } from "@/store/status.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export interface ValueDiff {
  normalizedResource: unknown | null;
  resource: unknown;
  domain: {
    id: AttributeValueId;
    value: unknown;
  };
}

export interface Reconciliation {
  updates: Record<AttributeValueId, unknown>;
  actions: string[];
  message?: string;
}

export interface Diff {
  // Key: Prop::path
  diff: Record<string, ValueDiff>;
  reconciliation: Reconciliation | null;
}

export const useReconciliationsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.selectedWorkspacePk;

  return addStoreHooks(
    defineStore(`w${workspacePk || "NONE"}/reconciliations`, {
      state: () => ({
        diffsByComponentId: null as Record<ComponentId, Diff> | null,
      }),
      actions: {
        async LOAD_RECONCILIATIONS() {
          return new ApiRequest<{ diffs: Record<ComponentId, Diff> }>({
            url: "/component/resource_domain_diff",
            params: { visibility_change_set_pk: nilId() },
            onSuccess: ({ diffs }) => {
              this.diffsByComponentId = diffs;
            },
          });
        },
        async ALTER_SIMULATION(componentIds: ComponentId[]) {
          if (!this.diffsByComponentId) return;
          const attributeValues: Record<AttributeValueId, unknown> = {};
          for (const componentId of componentIds) {
            const diffs = this.diffsByComponentId[componentId];
            if (!diffs?.reconciliation) continue;

            for (const [id, value] of Object.entries(
              diffs.reconciliation.updates,
            )) {
              attributeValues[id] = value;
            }
          }

          return new ApiRequest<{ success: true }>({
            method: "post",
            url: "/component/alter_simulation",
            params: { visibility_change_set_pk: nilId(), attributeValues },
            onSuccess: (_response) => {
              this.LOAD_RECONCILIATIONS();
            },
          });
        },
      },
      async onActivated() {
        this.LOAD_RECONCILIATIONS();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `workspace/${workspacePk}/head`, [
          {
            eventType: "ChangeSetWritten",
            callback: (writtenChangeSetId) => {
              if (writtenChangeSetId !== nilId()) return;
              this.LOAD_RECONCILIATIONS();
            },
          },
          {
            eventType: "ChangeSetApplied",
            callback: () => {
              this.LOAD_RECONCILIATIONS();
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
