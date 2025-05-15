<template>
  <div
    :class="
      clsx(
        'group',
        'flex flex-row gap-xs items-center',
        'p-2xs max-w-24 h-10 text-sm border cursor-pointer',
        themeClasses(
          'border-neutral-400 hover:border-action-500',
          'border-neutral-600 hover:border-action-300',
        ),
        selected
          ? themeClasses('bg-action-100', 'bg-action-800')
          : themeClasses('bg-neutral-100', 'bg-neutral-900'),
      )
    "
    :style="
      filter.color
        ? `border-left-color: ${filter.color}; border-radius: 0 0.25rem 0.25rem 0; border-left-width: 2px`
        : 'border-radius: 0.25rem'
    "
  >
    <Icon :name="filter.icon" class="flex-none" />
    <TruncateWithTooltip
      :class="
        themeClasses(
          'group-hover:text-action-500',
          'group-hover:text-action-300',
        )
      "
    >
      {{ filter.name }}
    </TruncateWithTooltip>
    <PillCounter :count="filter.count" />
  </div>
</template>

<script setup lang="ts">
import {
  Icon,
  PillCounter,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";
import { AssetFilter } from "./AddComponentModal.vue";

defineProps({
  filter: { type: Object as PropType<AssetFilter>, required: true },
  selected: { type: Boolean },
});
</script>
