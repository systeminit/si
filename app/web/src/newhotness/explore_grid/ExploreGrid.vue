<template>
  <AttributePanelBulk
    v-if="bulkEditing"
    @close="() => $emit('bulkDone')"
    @deselect="(idx) => $emit('childDeselect', idx)"
  />
  <div
    v-else
    data-testid="explore-grid"
    class="w-full relative flex flex-col"
    :style="{
      ['overflow-anchor']: 'none',
      height: `${virtualListHeight}px`,
    }"
  >
    <ExploreGridRow
      v-for="row in componentRowsVirtualItemsList"
      :key="`${row.key}`"
      data-testid="component-tile"
      :data-index="row.index"
      :class="clsx('absolute top-0 left-0 w-full')"
      :style="{
        height: `${rowHeights[row.index]}px`,
        transform: `translateY(${row.start}px)`,
      }"
      :row="gridRows[row.index]!"
      @childClicked="(e, c, idx) => $emit('childClicked', e, c, idx)"
      @childSelect="(idx) => $emit('childSelect', idx)"
      @childDeselect="(idx) => $emit('childDeselect', idx)"
      @childHover="(componentId) => $emit('childHover', componentId)"
      @childUnhover="(componentId) => $emit('childUnhover', componentId)"
      @clickCollapse="clickCollapse"
      @componentNavigate="
        (componentId) => $emit('componentNavigate', componentId)
      "
      @unpin="(componentId) => $emit('unpin', componentId)"
      @resetFilter="$emit('resetFilter')"
    />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, inject, watch } from "vue";
import * as _ from "lodash-es";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { ComponentId } from "@/api/sdf/dal/component";
import ExploreGridRow, { ExploreGridRowData } from "./ExploreGridRow.vue";
import { GRID_TILE_HEIGHT } from "./ExploreGridTile.vue";
import AttributePanelBulk from "./AttributePanelBulk.vue";
import { assertIsDefined, ExploreContext } from "../types";

const exploreContext = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(exploreContext);

const props = defineProps<{
  bulkEditing: boolean;
  gridRows: ExploreGridRowData[];
  scrollRef: HTMLDivElement | undefined;
}>();

const treatAsMultipleSections = computed(
  () =>
    exploreContext.hasMultipleSections.value ||
    exploreContext.gridMode.value.mode === "defaultSubscriptions",
);

const GRID_TILE_GAP = 16; // this is being used for both the X and Y gap

const clickCollapse = (title: string, collapsed: boolean) => {
  emit("collapse", title, collapsed);
};

const componentRowsVirtualItemsList = computed(() =>
  virtualList.value.getVirtualItems(),
);

// Rows need a unique item key, so the virtualizer internal watcher knows when to recompute sizes
const getItemKey = (rowIndex: number) => {
  const row = props.gridRows[rowIndex];
  if (!row) return rowIndex;

  switch (row.type) {
    case "header":
      return `header-${row.title}`;
    case "defaultSubHeader":
      return `defaultSubHeader-${row.subKey}`;
    case "contentRow":
      if (
        !treatAsMultipleSections.value &&
        rowIndex === props.gridRows.length - 1
      )
        return `contentRow-final-${rowIndex}`;
      else return `contentRow-${rowIndex}`;
    case "emptyRow":
      return `emptyContentRow-${rowIndex}`;
    case "filteredCounterRow":
      return `filteredCounterRow-${rowIndex}`;
    case "footer":
    default:
      return `footer-${rowIndex}`;
  }
};

const GROUP_HEADER_HEIGHT = 50;
const GROUP_FOOTER_HEIGHT = 10;
const GROUP_FILTERED_ROW_HEIGHT = 40;

const rowHeights = computed(() => {
  return props.gridRows.map((row, index) => {
    switch (row.type) {
      case "header":
      case "defaultSubHeader":
        return GROUP_HEADER_HEIGHT;
      case "contentRow":
        if (
          !treatAsMultipleSections.value &&
          index === props.gridRows.length - 1
        ) {
          return GRID_TILE_HEIGHT;
        } else {
          return GRID_TILE_HEIGHT + GRID_TILE_GAP;
        }
      case "pinnedContentRow":
        return GROUP_HEADER_HEIGHT;
      case "emptyRow":
        return GRID_TILE_HEIGHT + GRID_TILE_GAP;
      case "filteredCounterRow":
        return GROUP_FILTERED_ROW_HEIGHT;
      default:
        return GROUP_FOOTER_HEIGHT;
    }
  });
});

const virtualizerOptions = computed(() => ({
  count: props.gridRows.length,
  // `virtualizerLanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
  // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
  // Our grid is based on the minimum tile width... so how many tiles can we fit?
  // thats the value of `virtualizerLanes`
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  getScrollElement: () => props.scrollRef!,
  estimateSize: (i: number) => rowHeights.value[i] ?? 0,
  // The item key is essential for reactivity and was found by @vbustamante and his valiant
  // efforts. Without this, the virtualizer will not re-compute, even with new components.
  getItemKey: (i: number) => getItemKey(i),
  overscan: 4,
}));

const virtualList = useVirtualizer(virtualizerOptions);

const virtualListHeight = computed(() => virtualList.value.getTotalSize());

// Got through all the rows, save the id of the latest contentRow.
// If it's initialId is bigger than the id we're looking for, the previous one was the target row,
// otherwise, contentRow idx is the target.
// This is extra  complicated sinc ewe need to skip headers and footerss
const getRowIndexByGridTileIndex = (idx: number) => {
  let lastValidRowIndex = 0;

  for (let rowIndex = 0; rowIndex < props.gridRows.length; rowIndex++) {
    const row = props.gridRows[rowIndex];

    if (row?.type === "contentRow") {
      if (row.chunkInitialId > idx) {
        return lastValidRowIndex;
      } else {
        lastValidRowIndex = rowIndex;
      }
    }
  }

  return lastValidRowIndex;
};

// SCROLL BEHAVIOR
const scrollCurrentTileIntoView = () => {
  // don't scroll if the index is out of bounds
  if (
    exploreContext.focusedComponentIdx.value === undefined ||
    exploreContext.focusedComponentIdx.value < 0 ||
    exploreContext.focusedComponentIdx.value >
      exploreContext.allVisibleComponents.value.length - 1
  )
    return;

  // otherwise use the virtualizer to scroll
  // so that even if the DOM element doesn't exist
  // it will still work!
  virtualList.value.scrollToIndex(
    getRowIndexByGridTileIndex(exploreContext.focusedComponentIdx.value),
    { behavior: "smooth" },
  );
};

watch([() => exploreContext.focusedComponentIdx], scrollCurrentTileIntoView);

const emit = defineEmits<{
  (e: "bulkDone"): void;
  (e: "unpin", componentId: ComponentId): void;
  (e: "childSelect", componentIdx: number): void;
  (e: "childDeselect", componentIdx: number): void;
  (e: "childHover", componentId: ComponentId): void;
  (e: "childUnhover", componentId: ComponentId): void;
  (e: "collapse", title: string, collapsed: boolean): void;
  (e: "componentNavigate", componentId: ComponentId): void;
  (e: "resetFilter"): void;
  (
    e: "childClicked",
    event: MouseEvent,
    componentId: ComponentId,
    componentIdx: number,
  ): void;
}>();
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
