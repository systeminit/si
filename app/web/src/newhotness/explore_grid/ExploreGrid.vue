<template>
  <section>
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
        :class="clsx('absolute top-0 left-0 w-full')"
        :style="{
          height: `${gridRows[row.index]?.type === 'header' ? GROUP_HEADER_HEIGHT : GRID_TILE_HEIGHT}px`,
          transform: `translateY(${row.start}px)`,
        }"
      >
        <ExploreGridRow
          :lanesCount="virtualizerLanes"
          :row="gridRows[row.index]!"
          :focusedComponentId="focusedComponentId"
          @childHover="(c) => hover(getGridTileIndexByComponentId(c))"
          @childUnhover="(c) => unhover(getGridTileIndexByComponentId(c))"
          @childLeftClick="(e, c) => $emit('childLeftClick', e, c)"
          @childRightClick="(e, c) => $emit('childRightClick', e, c)"
        />
      </div>
    </div>
  </section>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import ComponentGridTile, { GRID_TILE_HEIGHT } from "../ComponentGridTile.vue";
import ExploreGridRow, { ExploreGridRowData } from "./ExploreGridRow.vue";
import { computed, ref, reactive, watch } from "vue";
import * as _ from "lodash-es";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import {
  KeyDetails,
  keyEmitter,
  windowResizeEmitter,
  windowWidthReactive,
} from "../logic_composables/emitters";
import {
  themeClasses,
  VormInput,
  VButton,
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
} from "@si/vue-lib/design-system";
import ComponentContextMenu from "../ComponentContextMenu.vue";
import { tw } from "@si/vue-lib";
import { useRouter, useRoute } from "vue-router";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { ComponentId } from "@/api/sdf/dal/component";

type ControlScheme = "v1" | "v2";
const CONTROL_SCHEME: ControlScheme = "v2" as ControlScheme;

const MIN_GRID_TILE_WIDTH = 250;
const GRID_TILE_GAP = 16;

const props = defineProps<{
  components: Record<string, ComponentInList[]>;
  scrollRef: HTMLDivElement | undefined; // Reference to parent element
  focusedComponentId?: ComponentId;
  focusGridPosition?: number;
  selectedComponentIdx?: number;
}>();

const allComponents = computed(() => {
  let components: ComponentInList[] = [];
  for (const title in props.components) {
    const newComponents = props.components[title] ?? [];
    components = [...components, ...newComponents];
  }

  return components;
});

const componentsById = computed(() =>
  Object.fromEntries(allComponents.value.map((c) => [c.id, c])),
);

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

// NOTE(nick,victor): this is probably broken!
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

// Grid rows

const gridRows = computed(() => {
  const rows: ExploreGridRowData[] = [];
  let chunkIndex = 0;
  for (const groupName in props.components) {
    const components = props.components[groupName];
    if (!components) continue;

    const count = components.length;

    rows.push({
      type: "header",
      title: groupName,
      count,
    });

    const componentChunks = _.chunk(components, virtualizerLanes.value);

    for (const components of componentChunks) {
      rows.push({
        type: "contentRow",
        components,
        chunkIndex: chunkIndex++,
      });
    }
  }

  return rows;
});

const filteredComponentRows = computed(() => {
  return _.chunk(allComponents.value, virtualizerLanes.value);
});

const componentRowsVirtualItemsList = computed(() =>
  virtualList.value.getVirtualItems(),
);

const GROUP_HEADER_HEIGHT = 50;

const virtualizerOptions = computed(() => ({
  count: gridRows.value.length,
  // `virtualizerLanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
  // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
  // Our grid is based on the minimum tile width... so how many tiles can we fit?
  // thats the value of `virtualizerLanes`
  getScrollElement: () => props.scrollRef!,
  estimateSize: (i: number) => {
    const row = gridRows.value[i];
    if (row && "title" in row) {
      return GROUP_HEADER_HEIGHT;
    } else {
      return GRID_TILE_HEIGHT;
    }
  },
  overscan: 1,
}));

const virtualList = useVirtualizer(virtualizerOptions);

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

const router = useRouter();
const route = useRoute();

// const isSelected = (idx: number) =>
//   selectedComponentIds.has(props.components[idx]!.id);
// const isHovered = (idx: number) => selectorGridPosition.value === idx;
// const isFocused = (idx: number) =>
//   focusGridPosition.value === idx && focusedComponentId.value;

const getGridTileIndexByComponentId = (id: ComponentId) => {
  return allComponents.value.findIndex((component) => component.id === id);
};

const scrollCurrentTileIntoView = () => {
  // don't scroll if the index is out of bounds
  if (
    !allComponents.value ||
    !props.selectedComponentIdx ||
    props.selectedComponentIdx < 0 ||
    props.selectedComponentIdx > allComponents.value.length - 1
  )
    return;
  // otherwise use the virtualizer to scroll
  // so that even if the DOM element doesn't exist
  // it will still work!
  virtualList.value.scrollToIndex(
    getRowIndexByGridTileIndex(props.selectedComponentIdx),
    { behavior: "smooth" },
  );
};

watch([props.selectedComponentIdx], scrollCurrentTileIntoView);

defineExpose({ getGridTileByIndex });
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
