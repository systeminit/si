<template>
  <DelayedComponent>
    <div class="grow grid grid-rows-2">
      <!-- history skel -->
      <div class="overflow-hidden flex flex-col">
        <h3
          :class="
            clsx(
              'flex flex-row items-center',
              'sticky top-0 text-lg font-bold px-xs py-2',
              'skeleton-shimmer',
              themeClasses('bg-neutral-200', 'bg-neutral-900'),
            )
          "
        >
          <div class="w-4 h-4 mr-2 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"></div>
          <div class="w-16 h-4 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"></div>
        </h3>
        <div class="flex-1 p-xs space-y-1">
          <div v-for="n in 8" :key="n" class="h-8 bg-neutral-200 dark:bg-neutral-700 rounded skeleton-shimmer"></div>
        </div>
      </div>

      <!-- actions skel -->
      <div class="overflow-hidden flex flex-col">
        <h3
          :class="
            clsx(
              'flex flex-row items-center',
              'sticky top-0 text-lg font-bold px-xs py-2',
              'skeleton-shimmer',
              themeClasses('bg-neutral-200', 'bg-neutral-900'),
            )
          "
        >
          <div class="w-4 h-4 mr-2 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"></div>
          <div class="w-16 h-4 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"></div>
        </h3>
        <div class="flex-1 p-xs space-y-1">
          <div v-for="n in 8" :key="n" class="h-8 bg-neutral-200 dark:bg-neutral-700 rounded skeleton-shimmer"></div>
        </div>
      </div>
    </div>
  </DelayedComponent>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import DelayedComponent from "@/newhotness/layout_components/DelayedComponent.vue";
</script>
