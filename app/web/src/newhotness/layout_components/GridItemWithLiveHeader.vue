<template>
  <div
    :class="
      clsx(
        'flex flex-col rounded overflow-hidden border',
        themeClasses(
          'bg-neutral-200 border-neutral-300',
          'bg-neutral-800 border-neutral-600',
        ),
      )
    "
  >
    <div class="flex flex-row items-center justify-between px-sm py-2xs">
      <h2 class="text-sm font-medium pt-xs pb-xs">{{ title }}</h2>
      <!-- Live updating indicator -->
      <div
        v-if="live"
        class="grow flex flex-row items-center text-xs text-action-400 ml-2"
      >
        <Icon name="loader" size="xs" class="mr-2xs" />
        Live
      </div>
    </div>
    <div
      :class="
        clsx(
          'overflow-auto flex-1 text-sm',
          themeClasses('bg-white', 'bg-[#0d1117]'),
        )
      "
    >
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps<{
  live: boolean;
  title: string;
}>();
</script>
