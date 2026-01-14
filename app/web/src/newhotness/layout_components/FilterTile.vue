<template>
  <div
    :class="
      clsx(
        'group',
        'flex flex-row gap-xs items-center',
        'p-2xs max-w-24 h-10 text-sm border cursor-pointer',
        themeClasses(
          'border-neutral-400 hover:border-action-500 active:bg-action-500',
          'border-neutral-600 hover:border-action-300 active:bg-action-300',
        ),
        selected ? themeClasses('bg-action-100', 'bg-action-800') : themeClasses('bg-neutral-100', 'bg-neutral-900'),
      )
    "
    :style="
      color
        ? `border-left-color: ${color}; border-radius: 0 0.25rem 0.25rem 0; border-left-width: 2px`
        : 'border-radius: 0.25rem'
    "
  >
    <Icon v-if="icon" :name="icon" class="flex-none" />
    <TruncateWithTooltip
      :class="
        clsx('group-active:text-shade-0', themeClasses('group-hover:text-action-500', 'group-hover:text-action-300'))
      "
    >
      {{ label }}
    </TruncateWithTooltip>
    <PillCounter v-if="count !== undefined" :count="count" />
  </div>
</template>

<script setup lang="ts">
import { Icon, IconNames, PillCounter, themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps<{
  count?: number;
  color?: string;
  label: string;
  icon?: IconNames;
  selected: boolean;
}>();
</script>
