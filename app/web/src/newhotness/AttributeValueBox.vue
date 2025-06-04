<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center gap-xs border w-fit px-3xs rounded-sm',
        themeClasses(
          'text-neutral-600 border-neutral-400 bg-neutral-100',
          'text-neutral-400 border-neutral-600 bg-neutral-900',
        ),
      )
    "
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
</script>
