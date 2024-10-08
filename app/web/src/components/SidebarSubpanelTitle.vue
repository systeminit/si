<template>
  <div
    :class="
      clsx(
        'flex text-neutral-500 dark:text-neutral-400 border-b items-center px-xs py-2xs gap-xs',
        !selectable && 'select-none',
        variant === 'title'
          ? 'dark:border-neutral-500'
          : 'border-neutral-200 dark:border-neutral-600',
      )
    "
  >
    <div class="flex-none empty:hidden">
      <slot name="icon">
        <Icon v-if="icon" :name="icon" />
      </slot>
    </div>

    <TruncateWithTooltip
      :class="
        clsx(
          'grow font-bold',
          variant === 'title'
            ? 'uppercase text-md leading-6'
            : 'text-sm break-words',
        )
      "
    >
      <slot name="label">{{ label }}</slot>
    </TruncateWithTooltip>
    <div class="flex-none empty:hidden">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Icon,
  IconNames,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";

export type SidebarSubpanelTitleVariant = "title" | "subtitle";

const props = defineProps({
  label: { type: String },
  icon: { type: String as PropType<IconNames> },
  variant: {
    type: String as PropType<SidebarSubpanelTitleVariant>,
    default: "title",
  },
  selectable: { type: Boolean },
});
</script>
