<template>
  <div
    :class="
      clsx(
        'border p-xs',
        themeClasses(
          'bg-destructive-100 border-destructive-600',
          'bg-newhotness-destructive border-destructive-400',
        ),
      )
    "
  >
    <!-- First row for icon and destructive text -->
    <div
      :class="
        clsx(
          'flex flex-row items-center gap-xs',
          themeClasses('text-destructive-600', 'text-destructive-200'),
        )
      "
    >
      <Icon name="x-circle" size="xs" />
      <span class="text-sm">{{ text }}</span>

      <!-- Slot for anything, but mainly a button for toggling the subtitle -->
      <div class="ml-auto">
        <slot name="right"></slot>
      </div>
    </div>

    <!-- Second row for a subtitle in neutral text -->
    <div
      v-if="showSubtitle && subtitle"
      class="flex flex-row items-center mt-xs"
    >
      <span>{{ subtitle }}</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps<{
  text: string;
  subtitle?: string;
  showSubtitle?: boolean;
}>();
</script>
