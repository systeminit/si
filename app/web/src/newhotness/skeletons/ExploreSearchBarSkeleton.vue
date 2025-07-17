<template>
  <DelayedComponent>
    <div class="flex flex-col gap-xs items-stretch">
      <div class="flex-none flex flex-row items-center gap-xs">
        <!-- View dropdown skeleton -->
        <div
          :class="
            clsx(
              'rounded min-w-[128px] h-8 skeleton-shimmer',
              themeClasses(
                'bg-neutral-200 border border-neutral-400',
                'bg-neutral-700 border border-neutral-600',
              ),
            )
          "
        ></div>

        <!-- search bar -->
        <div
          :class="
            clsx(
              'rounded grow h-8 skeleton-shimmer flex items-center px-2 gap-2',
              themeClasses(
                'bg-neutral-200 border border-neutral-400',
                'bg-neutral-700 border border-neutral-600',
              ),
            )
          "
        ></div>
      </div>
      <div class="flex-none flex flex-row items-center gap-xs justify-between">
        <!-- grid/map toggle -->
        <div class="flex flex-row gap-1">
          <div
            :class="
              clsx(
                'px-3 py-1 rounded h-8 skeleton-shimmer min-w-[128px]',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
          ></div>
        </div>

        <div class="flex flex-row gap-xs">
          <!-- group by dropdown skeleton -->
          <div
            :class="
              clsx(
                'rounded min-w-[100px] h-8 skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
          ></div>

          <!-- sort by dropdown skeleton -->
          <div
            :class="
              clsx(
                'rounded min-w-[100px] h-8 skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
          ></div>
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
