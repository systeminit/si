<template>
  <div
    class="w-full relative flex flex-col"
    :style="{
              ['overflow-anchor']: 'none',
              height: `${virtualListHeight}px`,
            }"
  >
    <div
      v-for="row in componentRowsVirtualItemsList"
      :key="`${row.key}`"
      :data-index="row.index"
      :class="
                clsx(
                  'flex flex-row items-center gap-sm',
                  'absolute top-0 left-0 w-full',
                )
              "
      :style="{
                height: `${GRID_TILE_HEIGHT}px`,
                transform: `translateY(${row.start}px)`,
              }"
    >
      <ComponentGridTile
        v-for="(component, columnIndex) in filteredComponentRows[
                  row.index
                ]"
        ref="componentGridTileRefs"
        :key="component.id"
        :data-index="row.index * virtualizerLanes + columnIndex"
        :component="component"
        class="flex-1"
        :class="
                  clsx(tileClasses(row.index * virtualizerLanes + columnIndex))
                "
        @mouseenter="hover(row.index * virtualizerLanes + columnIndex)"
        @mouseleave="
                  unhover(row.index * virtualizerLanes + columnIndex)
                "
        @click.stop.left="(e) => componentClicked(e, component.id)"
        @click.stop.right="(e) => componentClicked(e, component.id)"
      />
      <!-- this fills in any extra spots in the last row -->
      <div
        v-for="emptySpot in virtualizerLanes - filteredComponentRows[row.index]!.length"
        :key="emptySpot"
        class="flex-1"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import ComponentGridTile, { GRID_TILE_HEIGHT } from "./ComponentGridTile.vue";
import {computed, ref} from "vue";
import * as _ from "lodash-es";
import {ComponentInList} from "@/workers/types/entity_kind_types";
import {
  KeyDetails,
  keyEmitter,
  windowResizeEmitter,
  windowWidthReactive,
} from "./logic_composables/emitters";
import {useVirtualizer} from "@tanstack/vue-virtual";

const MIN_GRID_TILE_WIDTH = 250;
const GRID_TILE_GAP = 16;

const props = defineProps<{
  components: ComponentInList[];
  scrollRef: HTMLDivElement; // Reference to parent element
  enableKeyboardControls?: boolean;
}>();

function getScrollbarWidth(): number {
  const temp = document.createElement("div");
  const inner = document.createElement("div");

  temp.style.visibility = "hidden";
  temp.style.overflow = "scroll";
  document.body.appendChild(temp);
  temp.appendChild(inner);

  const scrollbarWidth = temp.offsetWidth - inner.offsetWidth;
  temp.parentNode?.removeChild(temp);

  return scrollbarWidth;
}

const componentGridTileRefs = ref<InstanceType<typeof ComponentGridTile>[]>();
const componentGridTileElsSorted = computed(() => {
  if (!componentGridTileRefs.value) {
    return [];
  } else {
    return componentGridTileRefs.value
      .map((tileRef) => tileRef.$el)
      .sort((a, b) => a.dataset.index - b.dataset.index);
  }
});
const getGridTileByIndex = (idx: number) => {
  if (componentGridTileRefs.value) {
    const tile = componentGridTileRefs.value.find((t) => {
      return Number(t.$el.dataset.index) === idx;
    });
    return tile;
  }
  return undefined;
};


const virtualizerOptions = computed(() => {
  const options = {
    count: filteredComponentRows.value.length,
    // `virtualizerLanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
    // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
    // Our grid is based on the minimum tile width... so how many tiles can we fit?
    // thats the value of `virtualizerLanes`
    getScrollElement: () => props.scrollRef!,
    estimateSize: () => MIN_GRID_TILE_WIDTH,
    overscan: 3,
  };
  return options;
});

const virtualList = useVirtualizer(virtualizerOptions);


// This computes the expected number of components in a row based on the width of the scroll area
const virtualizerLanes = computed(() => {
  // We need to force a recompute of this value when the screen is resized
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  windowWidthReactive.value;

  // We also need to force a recompute of this value if the number of tiles changes
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  componentGridTileRefs.value;

  // Our grid is based on the minimum tile width... so how many tiles can we fit?
  let newLanes = 0;
  let availableSpace = props.scrollRef?.getBoundingClientRect().width ?? 0;
  if (
    props.scrollRef &&
    props.scrollRef.scrollHeight > props.scrollRef.clientHeight
  ) {
    // need to account for the width of the scrollbar!
    availableSpace -= getScrollbarWidth();
  }
  while (availableSpace > 0) {
    availableSpace -= MIN_GRID_TILE_WIDTH; // width of one grid tile
    if (availableSpace > 0) {
      newLanes++;
    }
    availableSpace -= GRID_TILE_GAP; // gap between grid tiles
  }
  return newLanes;
});


const filteredComponentRows = computed(() =>
  _.chunk(props.components, virtualizerLanes.value),
);


const componentRowsVirtualItemsList = computed(() =>
  virtualList.value.getVirtualItems(),
);



const virtualListHeight = computed(() => virtualList.value.getTotalSize());

const getRowIndexByGridTileIndex = (idx: number) =>
  Math.floor(idx / virtualizerLanes.value);

// This computes the rendered number of components in a row as seen directly in the DOM
const lanes = computed(() => {
  // We need to force a recompute of this value when the screen is resized
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  windowWidthReactive.value;

  // Can't calculate the amount of grid tiles per row if we don't have any grid tiles loaded yet!
  const componentGridTileYPositions = componentGridTileElsSorted.value.map(
    (el) => el.getBoundingClientRect().y,
  );
  if (componentGridTileYPositions.length === 0) return 0;

  let newLanes = 1;
  const firstLaneY = componentGridTileYPositions[0];

  while (
    componentGridTileYPositions[newLanes] === firstLaneY &&
    newLanes < componentGridTileYPositions.length
    ) {
    newLanes++;
  }
  return newLanes;
});




</script>

<style lang="css" scoped>
section.grid.explore {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 100%;
  grid-template-areas: "main right";
}

section.grid.map {
  grid-template-columns: 100%;
  grid-template-rows: 100%;
  grid-template-areas: "main";
}

div.main {
  grid-area: "main";
}
div.right {
  grid-area: "right";
}
</style>
