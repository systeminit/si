<template>
  <span
    :class="
      clsx(
        'text-pill',
        'border rounded-sm font-normal whitespace-nowrap',
        tighter ? 'leading-snug tracking-tighter px-3xs' : 'py-3xs px-2xs',
        mono && 'font-mono pt-3xs',
        {
          simple: '',
          key: themeClasses(
            'border-neutral-400 bg-neutral-100',
            'border-neutral-500 bg-neutral-800',
          ),
          key2: themeClasses(
            'border-neutral-600 bg-neutral-100',
            'border-neutral-400 bg-neutral-800',
          ),
          component: themeClasses(
            'border-neutral-400 bg-neutral-100',
            'border-neutral-600 bg-neutral-900',
          ),
          destructive: themeClasses(
            'border-destructive-500 bg-destructive-100 text-black',
            'border-destructive-600 bg-destructive-900 text-white',
          ),
          warning: themeClasses(
            'border-warning-500 bg-warning-100 text-black',
            'border-warning-600 bg-warning-900 text-white',
          ),
          success: themeClasses(
            'border-success-500 bg-neutral-100 text-black',
            'border-success-600 bg-neutral-900 text-white',
          ),
        }[variant],
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

export type TextPillVariant =
  | "simple"
  | "key"
  | "key2"
  | "component"
  | "destructive"
  | "warning"
  | "success";

defineProps({
  tighter: { type: Boolean },
  variant: { type: String as PropType<TextPillVariant>, default: "simple" },
  mono: { type: Boolean },
});
</script>
