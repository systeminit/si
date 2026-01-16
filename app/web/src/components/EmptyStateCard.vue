<template>
  <div :class="clsx('flex flex-col items-center gap-xs', marginClass)">
    <div class="w-64 flex flex-col items-center">
      <EmptyStateIcon :name="iconName" />
    </div>
    <div class="text-lg font-medium text-center text-neutral-600 dark:text-neutral-300">
      {{ primaryText }}
    </div>
    <div class="text-sm italic text-neutral-500 dark:text-neutral-400 text-center">
      {{ secondaryText }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import { SpacingSizes } from "@si/vue-lib/design-system";
import clsx from "clsx";
import EmptyStateIcon, { EMPTY_STATE_ICON_NAMES } from "./EmptyStateIcon.vue";

const props = defineProps({
  iconName: {
    type: String as PropType<EMPTY_STATE_ICON_NAMES>,
    required: true,
  },
  primaryText: { type: String, required: true },
  secondaryText: { type: String, required: true },
  margin: { type: String as PropType<SpacingSizes>, default: "sm" },
});

const marginClass = computed(
  () =>
    ({
      none: "m-0",
      "2xs": "m-2xs",
      xs: "m-xs",
      sm: "m-sm",
      md: "m-md",
      lg: "m-lg",
      xl: "m-xl",
      "2xl": "m-2xl",
    }[props.margin]),
);
</script>
