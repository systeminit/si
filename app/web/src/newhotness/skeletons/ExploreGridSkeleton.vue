<template>
  <section
    :class="clsx('grid h-full explore', themeClasses('bg-white', 'bg-black'))"
  >
    <div
      :class="
        clsx(
          'main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]',
          themeClasses('bg-white', 'bg-black'),
        )
      "
    >
      <!-- view dropdown / search bar -->
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
                themeClasses('bg-neutral-200', 'bg-neutral-700'),
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

      <!-- component grid skeleton -->
      <div
        :class="
          clsx(
            'w-full grid gap-4 flex-1 pr-6',
            themeClasses('bg-white', 'bg-black'),
          )
        "
        :style="{
          'grid-template-columns': `repeat(${columnsCount}, minmax(0, 1fr))`,
        }"
      >
        <GridTileSkeleton v-for="n in skeletonCount" :key="n" />
      </div>
    </div>

    <!-- Right panel -->
    <div
      :class="
        clsx(
          'right flex flex-col border-l',
          themeClasses(
            'bg-neutral-100 border-neutral-400',
            'bg-neutral-800 border-neutral-600',
          ),
        )
      "
    >
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
            <div
              class="w-4 h-4 mr-2 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"
            ></div>
            <div
              class="w-16 h-4 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"
            ></div>
          </h3>
          <div class="flex-1 p-xs space-y-1">
            <div
              v-for="n in 8"
              :key="n"
              class="h-8 bg-neutral-200 dark:bg-neutral-700 rounded skeleton-shimmer"
            ></div>
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
            <div
              class="w-4 h-4 mr-2 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"
            ></div>
            <div
              class="w-16 h-4 bg-neutral-300 dark:bg-neutral-700 rounded skeleton-shimmer"
            ></div>
          </h3>
          <div class="flex-1 p-xs space-y-1">
            <div
              v-for="n in 8"
              :key="n"
              class="h-8 bg-neutral-200 dark:bg-neutral-700 rounded skeleton-shimmer"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import GridTileSkeleton from "./GridTileSkeleton.vue";
import {
  windowWidthReactive,
  windowHeightReactive,
} from "../logic_composables/emitters";

const MIN_GRID_TILE_WIDTH = 250;
const GRID_TILE_GAP = 16;
const GRID_TILE_HEIGHT = 233;

const columnsCount = computed(() => {
  let lanes = 0;
  // account for right panel (30% of width),
  // horizontal margins (24px),
  // and additional right padding (24px) = 48px
  let availableSpace = windowWidthReactive.value * 0.7 - 48;

  while (availableSpace > 0) {
    availableSpace -= MIN_GRID_TILE_WIDTH;
    if (availableSpace > 0) {
      lanes++;
    }
    availableSpace -= GRID_TILE_GAP;
  }

  return Math.max(1, lanes); // Ensure at least 1 column
});

const rowsCount = computed(() => {
  // account for top controls (2 rows ~32px each), footer (~48px), padding, and margins
  const availableHeight = windowHeightReactive.value - 150;
  return Math.max(
    3,
    Math.ceil(availableHeight / (GRID_TILE_HEIGHT + GRID_TILE_GAP)),
  );
});

const skeletonCount = computed(() => columnsCount.value * rowsCount.value);
</script>

<style lang="css" scoped>
section.grid.explore {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 100%;
  grid-template-areas: "main right";
}

div.main {
  grid-area: main;
}

div.right {
  grid-area: right;
}
</style>
