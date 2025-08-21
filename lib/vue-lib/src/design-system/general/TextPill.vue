<template>
  <span
    :class="
      clsx(
        'text-pill',
        'border rounded-sm font-normal',
        tighter ? 'leading-snug tracking-tighter px-3xs' : 'py-3xs px-2xs',
        mono && 'font-mono pt-3xs',
        variant === 'key' &&
          themeClasses(
            'border-neutral-400 bg-neutral-100',
            'border-neutral-500 bg-neutral-800',
          ),
        variant === 'key2' &&
          themeClasses(
            'border-neutral-600 bg-neutral-100',
            'border-neutral-400 bg-neutral-800',
          ),
        variant === 'component' &&
          themeClasses(
            'border-neutral-400 bg-neutral-100',
            'border-neutral-600 bg-neutral-900',
          ),
      )
    "
  >
    <slot />
  </span>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import clsx from "clsx";
import { themeClasses } from "../utils/theme_tools";

export type TextPillVariant = "simple" | "key" | "key2" | "component";

defineProps({
  tighter: { type: Boolean },
  variant: { type: String as PropType<TextPillVariant>, default: "simple" },
  mono: { type: Boolean },
});
</script>
