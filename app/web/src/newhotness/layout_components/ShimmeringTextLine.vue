<template>
  <div :class="clsx('skeleton-shimmer flex-none rounded-sm', themeClasses('bg-neutral-400', 'bg-neutral-700'))" />
</template>

<script setup lang="ts">
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
</script>
