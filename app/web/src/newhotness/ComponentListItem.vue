<template>
  <div
    tabindex="0"
    :data-list-item-component-id="component.id"
    :class="
      clsx(
        'absolute top-0 left-0 w-full',
        'flex flex-row items-center gap-xs p-2xs text-sm h-8',
        'border border-transparent rounded-sm cursor-pointer outline-none',
        '[&>span]:min-w-0',
        themeClasses('hover:border-action-500', 'hover:border-action-300'),
        selected && themeClasses('bg-action-200', 'bg-action-900'),
      )
    "
  >
    <StatusIndicatorIcon class="flex-none" type="diff" :status="status" />
    <TextPill
      tighter
      variant="component"
      :class="
        clsx(
          themeClasses(
            'text-newhotness-greenlight',
            'text-newhotness-greendark',
          ),
          'max-w-fit flex-1',
        )
      "
    >
      <TruncateWithTooltip>{{ component.schemaName }}</TruncateWithTooltip>
    </TextPill>
    <TextPill
      tighter
      variant="component"
      :class="
        clsx(
          'max-w-fit flex-1',
          themeClasses(
            'text-newhotness-purplelight',
            'text-newhotness-purpledark',
          ),
        )
      "
    >
      <TruncateWithTooltip>{{ component.name }}</TruncateWithTooltip>
    </TextPill>
    <StatusIndicatorIcon
      class="ml-auto flex-none"
      type="qualification"
      :status="qualificationStatus"
    />
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
  ComponentDiffStatus,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { getQualificationStatus } from "./ComponentTileQualificationStatus.vue";

const props = defineProps<{
  component: ComponentInList | BifrostComponent;
  status: ComponentDiffStatus;
  selected?: boolean;
}>();

const qualificationStatus = computed(() =>
  getQualificationStatus(props.component),
);
</script>
