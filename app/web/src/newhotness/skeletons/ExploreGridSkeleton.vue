<template>
  <DelayedComponent>
    <div
      :class="clsx('w-full grid gap-4 flex-1 pr-6')"
      :style="{
        'grid-template-columns': `repeat(${columnsCount}, minmax(0, 1fr))`,
      }"
    >
      <GridTileSkeleton v-for="n in skeletonCount" :key="n" />
    </div>
  </DelayedComponent>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import DelayedComponent from "@/newhotness/layout_components/DelayedComponent.vue";
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
