<template>
  <span
    :class="
      clsx(
        'border rounded-sm',
        tighter ? 'leading-snug px-3xs' : 'py-3xs px-2xs',
        themeClasses(
          'border-neutral-600 bg-neutral-100',
          'border-neutral-400 bg-neutral-800',
        ),
      )
    "
    ><slot
  /></span>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";

defineProps({
  tighter: { type: Boolean },
});
</script>
