<template>
  <ul
    v-if="edges.length > 0"
    :class="
      clsx(
        'flex flex-col gap-xs rounded border [&>li]:px-xs py-xs',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
  >
    <li
      :class="
        clsx(
          'text-lg font-bold border-b',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
    >
      {{ titleText }}
    </li>
    <div
      v-if="selectComponent"
      :class="
        clsx(
          'mx-xs px-sm flex flex-row items-center gap-sm',
          themeClasses('bg-neutral-300', 'bg-neutral-700'),
        )
      "
    >
      <TruncateWithTooltip class="py-sm grow">
        Add components to be managed by "{{ parentComponentName }}"
      </TruncateWithTooltip>
      <div
        :class="
          clsx(
            'min-w-[300px] flex flex-row items-center gap-xs',
            'h-lg p-xs ml-auto text-sm border font-mono cursor-text',
            themeClasses(
              'text-shade-100 bg-shade-0 border-neutral-400',
              'text-shade-0 bg-shade-100 border-neutral-600',
            ),
          )
        "
      >
        <Icon name="search" size="sm" class="flex-none" />
        <div class="grow">Find and select components</div>
        <Icon name="chevron--down" size="sm" class="flex-none" />
      </div>
    </div>
    <ManagementConnectionCard
      v-for="edge in edges"
      :key="edge.key"
      :edge="edge"
    />
  </ul>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import clsx from "clsx";
import {
  Icon,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { SimpleConnection } from "./layout_components/ConnectionLayout.vue";
import ManagementConnectionCard from "./ManagementConnectionCard.vue";

defineProps({
  edges: { type: Array as PropType<SimpleConnection[]>, required: true },
  titleText: { type: String, default: "Management Connections" },
  selectComponent: { type: Boolean },
  parentComponentName: { type: String },
});
</script>
