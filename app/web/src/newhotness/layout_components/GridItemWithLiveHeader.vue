<template>
  <div class="flex flex-col rounded bg-neutral-800 overflow-hidden">
    <div class="flex flex-row items-center justify-between px-sm py-2xs">
      <h2 class="text-sm font-medium">{{ title }}</h2>
      <!-- Live updating indicator -->
      <div
        v-if="live"
        class="grow flex flex-row items-center text-xs text-action-400 ml-2"
      >
        <Icon name="loader" size="xs" class="mr-2xs" />
        Live
      </div>
      <VButton
        icon="expand-row"
        tone="neutral"
        variant="ghost"
        size="xs"
        class="text-neutral-400"
        aria-label="Expand logs"
      />
    </div>
    <div class="overflow-auto flex-1 text-sm">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { VButton, Icon } from "@si/vue-lib/design-system";

defineProps<{
  live: boolean;
  title: string;
}>();
</script>
