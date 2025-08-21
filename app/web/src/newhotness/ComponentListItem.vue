<template>
  <div
    tabindex="0"
    :data-list-item-component-id="component.id"
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs text-sm',
        'border border-transparent rounded-sm cursor-pointer outline-none',
        '[&>span]:min-w-0',
        themeClasses('hover:border-action-500', 'hover:border-action-300'),
        selected && themeClasses('bg-action-200', 'bg-action-900'),
      )
    "
  >
    <StatusIndicatorIcon type="qualification" :status="qualificationStatus" />
    <TextPill
      tighter
      variant="component"
      :class="themeClasses('text-green-light-mode', 'text-green-dark-mode')"
    >
      <TruncateWithTooltip>{{ component.schemaName }}</TruncateWithTooltip>
    </TextPill>
    <TextPill tighter variant="component" class="text-purple">
      <TruncateWithTooltip>{{ component.name }}</TruncateWithTooltip>
    </TextPill>
  </div>
</template>

<script setup lang="ts">
import {
  TextPill,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import {
  BifrostComponent,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { getQualificationStatus } from "./ComponentTileQualificationStatus.vue";

const props = defineProps<{
  component: ComponentInList | BifrostComponent;
  selected?: boolean;
}>();

const qualificationStatus = computed(() =>
  getQualificationStatus(props.component),
);
</script>
