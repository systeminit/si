<template>
  <div class="flex flex-row items-center gap-2xs">
    <TruncateWithTooltip>{{ text }}</TruncateWithTooltip>
    <span
      v-if="showAsterisk"
      v-tooltip="`${text} is a required value`"
      :class="clsx('shrink-0 text-lg font-bold', themeClasses('text-destructive-600', 'text-destructive-300'))"
    >
      *
    </span>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";

defineProps<{
  text: string;
  showAsterisk?: boolean;
}>();
</script>
