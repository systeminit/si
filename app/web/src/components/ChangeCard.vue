<template>
  <div
    :class="
      clsx(
        'flex flex-row gap-xs items-center text-sm p-xs border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
  >
    <StatusIndicatorIcon type="change" :status="diff.status" tone="shade" />
    <div class="flex flex-col overflow-hidden">
      <div class="">
        <span v-if="diff.status === 'added'">Added</span>
        <span v-if="diff.status === 'deleted'">Removed</span>
        <span v-if="diff.status === 'modified'">Modified</span>
        {{ componentsStore.componentsById[diff.componentId]?.schemaName }}
      </div>
      <div class="text-neutral-400 truncate">
        {{ componentsStore.componentsById[diff.componentId]?.displayName }}
      </div>
      <div class="text-neutral-400 truncate">By: {{ diff.actor }}</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { useComponentsStore } from "@/store/components.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

export type DiffInfo = {
  componentId: string;
  status: ChangeStatus;
  updatedAt: string;
  actor: string;
};

const componentsStore = useComponentsStore();

defineProps({
  diff: { type: Object as PropType<DiffInfo>, required: true },
});
</script>
