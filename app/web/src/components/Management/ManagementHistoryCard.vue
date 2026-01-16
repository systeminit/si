<template>
  <li :class="clsx('border-b list-none', themeClasses('border-neutral-200', 'border-neutral-600'))">
    <div
      :class="
        clsx(
          'cursor-pointer flex flex-row items-center gap-xs p-2xs pl-4 border text-sm border-transparent',
          themeClasses('hover:border-action-500', 'hover:border-action-300'),
          selected
            ? themeClasses('bg-action-100 border-action-500', 'bg-action-900 border-action-300')
            : themeClasses('border-neutral-800', ''),
        )
      "
      @click="emit('clickItem', item, $event)"
    >
      <StatusIndicatorIcon type="management" :status="status" />

      <TruncateWithTooltip class="grow">{{ item.functionDisplayName ?? item.functionName }}</TruncateWithTooltip>

      <Timestamp
        :date="item.updatedAt"
        :timeClasses="themeClasses('text-neutral-500', 'text-neutral-400')"
        class="text-xs"
        dateClasses="font-bold"
        showTimeIfToday
        size="long"
      />

      <FuncRunTabDropdown :funcRunId="item.id" @menuClick="(id, slug) => emit('history', id, slug)" />
    </div>
  </li>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import { themeClasses, TruncateWithTooltip, Timestamp } from "@si/vue-lib/design-system";
import { funcRunStatus } from "@/store/func_runs.store";
import { ManagementHistoryItem } from "@/store/management_runs.store";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import FuncRunTabDropdown from "../FuncRunTabDropdown.vue";

const props = defineProps<{
  item: ManagementHistoryItem;
  selected?: boolean;
}>();

// We're hijacking the action status here since that's what we store in the FuncRun
const status = computed(() => funcRunStatus(props.item));

const emit = defineEmits<{
  (e: "history", id: string, tabSlug: string): void;
  (e: "clickItem", item: ManagementHistoryItem, event: MouseEvent): void;
}>();
</script>
