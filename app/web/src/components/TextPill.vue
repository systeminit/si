<template>
  <span
    :class="
      clsx(
        'border rounded-sm px-2xs',
        !noVerticalPadding && 'py-3xs',
        themeClasses(
          'border-neutral-400 bg-neutral-100',
          'border-neutral-500 bg-neutral-800',
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
  noVerticalPadding: { type: Boolean },
});
</script>
