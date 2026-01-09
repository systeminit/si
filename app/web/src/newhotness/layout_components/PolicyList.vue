<template>
  <EmptyStateCard
    v-if="policies.length === 0"
    iconName="no-changes"
    primaryText="No policies to view"
    secondaryText="Use the command line `si policy evaluate` to generate policy reports."
  />
  <ol v-else class="p-xs h-full overflow-y-auto">
    <li v-if="maxPages > 1" class="flex flex-row-reverse gap-xs items-center">
      <NewButton
        tooltip="Forward"
        tooltipPlacement="top"
        icon="chevron--right"
        tone="empty"
        :class="
          clsx(themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-600'))
        "
        @click="() => emits('pageForward')"
      />
      <span class="text-xs">{{ page }}</span>
      <NewButton
        tooltip="Back"
        tooltipPlacement="top"
        icon="chevron--left"
        tone="empty"
        :class="
          clsx(themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-600'))
        "
        @click="() => emits('pageBack')"
      />
    </li>
    <li
      v-for="p in policies"
      :key="p.id"
      :class="
        clsx(
          'p-xs cursor-pointer border mb-xs',
          themeClasses(
            'hover:border-action-500 border-neutral-200',
            'hover:border-action-300 border-neutral-600',
          ),
          'flex flex-row items-center text-xs gap-sm justify-between',
        )
      "
      @click="() => emits('select', p)"
    >
      <Icon
        class="shrink"
        size="xs"
        :name="p.result === 'Fail' ? 'triangle' : 'check-square'"
        :tone="p.result === 'Fail' ? 'destructive' : 'success'"
      />
      <span class="grow"
        ><TruncateWithTooltip>{{ p.name }}</TruncateWithTooltip></span
      >
      <span class="shrink">
        <Timestamp
          refresh
          size="normal"
          relative="standard"
          showTimeIfToday
          :date="p.createdAt"
        />
      </span>
    </li>
  </ol>
</template>

<script setup lang="ts">
import { clsx } from "clsx";
import {
  themeClasses,
  NewButton,
  Icon,
  Timestamp,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { Policy } from "../logic_composables/policy";
import EmptyStateCard from "../../components/EmptyStateCard.vue";

const props = defineProps<{
  policies: Policy[];
  page: number;
  maxPages: number;
}>();

const emits = defineEmits<{
  (e: "select", policy: Policy): void;
  (e: "pageBack"): void;
  (e: "pageForward"): void;
}>();
</script>
