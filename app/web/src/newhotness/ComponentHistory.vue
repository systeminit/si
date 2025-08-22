<template>
  <div v-if="auditLogs.length > 0" class="p-sm my-xs overflow-x-hidden">
    <ul class="space-y-md">
      <li
        v-for="auditLog in auditLogs"
        :key="auditLog.timestamp"
        class="flex flex-row items-center gap-xs"
      >
        <div class="w-2.5 h-2.5 rounded-full bg-neutral-500" />
        <div
          class="flex flex-col p-xs border rounded-sm border-neutral-600"
          @click="
            () => (expand[auditLog.timestamp] = !expand[auditLog.timestamp])
          "
        >
          <div class="flex flex-row gap-xs items-center">
            <TruncateWithTooltip>
              {{ auditLog.title }} {{ auditLog.entityType }}
            </TruncateWithTooltip>
            <Timestamp
              class="text-neutral-400"
              :date="auditLog.timestamp"
              relative="shorthand"
              enableDetailTooltip
              refresh
            />
            <Icon
              class="text-neutral-400"
              :name="
                expand[auditLog.timestamp] ? 'chevron--down' : 'chevron--left'
              "
            />
          </div>
          <div v-if="expand[auditLog.timestamp]" class="mt-xs">
            <CodeViewer :code="JSON.stringify(auditLog, null, 2)" />
          </div>
        </div>
      </li>
    </ul>
  </div>
  <EmptyState
    v-else
    class="p-lg"
    icon="component"
    text="No Direct Changes"
    secondaryText="There were no direct changes made to the component."
  />
</template>

<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, reactive } from "vue";
import {
  TruncateWithTooltip,
  Timestamp,
  Icon,
} from "@si/vue-lib/design-system";
import { ComponentId } from "@/api/sdf/dal/component";
import { AuditLog } from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { routes, useApi } from "./api_composables";
import EmptyState from "./EmptyState.vue";
import { useContext } from "./logic_composables/context";

const props = defineProps<{
  componentId: ComponentId;
}>();

const ctx = useContext();

const expand = reactive<Record<string, boolean>>({});

// TODO(nick): allow the user to load more.
const size = "100";

const auditLogsApi = useApi(ctx);
const auditLogsQuery = useQuery<AuditLog[]>({
  queryKey: ["auditlogs", ctx.changeSetId.value, props.componentId],
  queryFn: async () => {
    const call = auditLogsApi.endpoint<{
      logs: AuditLog[];
      canLoadMore: boolean;
    }>(routes.AuditLogsForComponent, { componentId: props.componentId });

    const response = await call.get(
      new URLSearchParams({
        size,
        sort_ascending: "false",
      }),
    );

    if (auditLogsApi.ok(response)) {
      return response.data.logs;
    }

    return [] as AuditLog[];
  },
});

// Filter out socket-related audit logs.
const auditLogs = computed(
  (): AuditLog[] =>
    auditLogsQuery.data.value?.filter(
      (auditLog) =>
        auditLog.kind !== "UpdateDependentOutputSocket" &&
        auditLog.kind !== "UpdateDependentInputSocket",
    ) ?? [],
);
</script>
