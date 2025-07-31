<template>
  <!-- without ref=scrollRef (which i dont need here) the component breaks -->
  <div v-if="bulkEditing" ref="scrollRef" class="grow min-h-0">
    <AttributePanelBulk
      :selectedComponents="selectedComponentsMap"
      @close="() => $emit('bulkDone')"
      @deselect="(idx) => $emit('childDeselect', idx)"
    />
  </div>
  <!--
    NOTE(nick,victor,wendy): we need both divs here for the virtualizer. The first div is the
    scrolling area itself (it will be whatever height fills the spot in the overall UI, which,
    at the time of writing, is a CSS grid). The second div is a wrapper that maintains the height
    of all of the virtualized rows, so that even when only a few rows are rendering, the scrollable
    area will not change. If you mess with this, it will break in ways YOU MAY OR MAY NOT NOTICE.
    BUYER BEWARE.
  -->
  <div
    v-else
    ref="scrollRef"
    class="scrollable grow"
    style="overflow-anchor: none"
  >
    <div
      data-testid="tile-container"
      class="w-full relative flex flex-col"
      :style="{
        ['overflow-anchor']: 'none',
        height: `${virtualListHeight}px`,
      }"
    >
      <ExploreGridRow
        v-for="row in componentRowsVirtualItemsList"
        :key="`${row.key}`"
        ref="exploreGridRowRefs"
        data-testid="component-tile"
        :data-index="row.index"
        :class="clsx('absolute top-0 left-0 w-full')"
        :style="{
          height: `${rowHeights[row.index]}px`,
          transform: `translateY(${row.start}px)`,
        }"
        :lanesCount="virtualizerLanes"
        :row="gridRows[row.index]!"
        :focusedComponentId="focusedComponent?.id"
        :selectedComponentIndexes="selectedComponentIndexes"
        :componentsWithFailedActions="componentsWithFailedActions"
        :componentsWithRunningActions="componentsWithRunningActions"
        :componentsPendingActionNames="componentsPendingActionNames"
        @childClicked="(e, c, idx) => $emit('childClicked', e, c, idx)"
        @childSelect="(idx) => $emit('childSelect', idx)"
        @childDeselect="(idx) => $emit('childDeselect', idx)"
        @childHover="(componentId) => $emit('childHover', componentId)"
        @childUnhover="(componentId) => $emit('childUnhover', componentId)"
        @clickCollapse="clickCollapse"
        @unpin="(componentId) => $emit('unpin', componentId)"
        @resetFilter="$emit('resetFilter')"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import { ComponentId } from "@/api/sdf/dal/component";
import { windowWidthReactive } from "../logic_composables/emitters";
import ExploreGridRow, { ExploreGridRowData } from "./ExploreGridRow.vue";
import ComponentCard from "../ComponentCard.vue";
import ExploreGridTile, { GRID_TILE_HEIGHT } from "./ExploreGridTile.vue";
import AttributePanelBulk from "./AttributePanelBulk.vue";

const props = defineProps<{
  bulkEditing: boolean;
  components: Record<string, ComponentInList[]>;
  focusedComponentIdx?: number;
  selectedComponentIndexes: Set<number>;
  componentsWithFailedActions: Set<ComponentId>;
  componentsWithRunningActions: Set<ComponentId>;
  componentsPendingActionNames: Map<
    ComponentId,
    Record<string, { count: number; hasFailed: boolean }>
  >;
  showFilteredCounter?: boolean;
  filteredCount?: number;
  totalCount?: number;
}>();

const MIN_GRID_TILE_WIDTH = 250;
const GRID_TILE_GAP = 16; // this is being used for both the X and Y gap

const scrollRef = ref<HTMLDivElement>();
const exploreGridRowRefs = ref<InstanceType<typeof ExploreGridRow>[]>();

const exploreGridComponentRefs = computed(() => {
  if (!exploreGridRowRefs.value) return [];

  const componentRefs: InstanceType<
    typeof ComponentCard | typeof ExploreGridTile
  >[] = [];

  for (const row of exploreGridRowRefs.value) {
    if (!row.exploreGridComponentRefs) continue;

    componentRefs.push(...row.exploreGridComponentRefs);
  }

  return componentRefs;
});

const allVisibleComponents = computed(() => {
  // this excludes components which are inside collapsed groups
  const components: ComponentInList[] = [];
  for (const row of gridRows.value) {
    if (row.type === "contentRow") {
      components.push(...row.components);
    }
  }
  return components;
});

const focusedComponent = computed(
  () => allVisibleComponents.value[props.focusedComponentIdx ?? -1],
);

const selectedComponentsMap = computed(() => {
  const selected: Record<number, ComponentInList> = {};

  props.selectedComponentIndexes.forEach((index) => {
    const component = allVisibleComponents.value[index];
    if (component) {
      selected[index] = component;
    }
  });

  return selected;
});
const selectedComponents = computed(() => {
  return Object.values(selectedComponentsMap.value);
});

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

const getGridComponentRefByIndex = (idx: number) => {
  if (!exploreGridComponentRefs.value) return undefined;

  return exploreGridComponentRefs.value.find(
    (t) => Number(t.$el.dataset.index) === idx,
  );
};

// The expected number of components in a row based on the width of the scroll area
const virtualizerLanes = computed(() => {
  // We need to force a recompute of this value when the screen is resized
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  windowWidthReactive.value;

  // We also need to force a recompute of this value if the number of tiles changes
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  exploreGridComponentRefs.value;

  // Our grid is based on the minimum tile width... so how many tiles can we fit?
  let newLanes = 0;
  let availableSpace = scrollRef.value?.getBoundingClientRect().width ?? 0;
  if (
    scrollRef.value &&
    scrollRef.value.scrollHeight > scrollRef.value.clientHeight
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

// This is how we show no headers when "group by" functionality is in use. This relies on the
// fact that using "group by" will create at least two groups. If you find yourself working on
// "group by", but only wanting to show one group, this is why you're not seeing any headers.
const hasMultipleSections = computed(() => _.keys(props.components).length > 1);

const gridRows = computed(() => {
  const rows: ExploreGridRowData[] = [];
  let dataIndex = 0;

  for (const groupName in props.components) {
    const components = props.components[groupName];
    if (!components) continue;

    // First, handle pinned components. They take up and entire row, so we can handle them upfront
    // without having to worry about chunking. We'll add a footer for each one.
    if (groupName === "Pinned") {
      for (const component of components) {
        rows.push({
          type: "pinnedContentRow",
          component,
          dataIndex,
        });
        dataIndex += 1;
        rows.push({
          type: "footer",
        });
      }

      // Move on after dealing with the pinned group.
      continue;
    }

    const count = components.length;
    let collapsed = collapseTracker.value[groupName];

    // Handle the very first time everything is loaded. We want empty sections to begin collapsed
    // and non-empty sections to be expanded by default. The "Unconnected" section should always
    // start collapsed.
    if (collapsed === undefined) {
      collapsed = count === 0 || groupName === "Unconnected";
    }

    if (hasMultipleSections.value) {
      rows.push({
        type: "header",
        title: groupName,
        count,
        collapsed,
      });
    }

    // Only populate the component rows if the header is not collapsed. Note that this removes them
    // from the virtualizer. We may eventually want to "hide" components instead to keep them
    // virtualized (e.g. "zero height").
    if (!collapsed) {
      const componentChunks = _.chunk(components, virtualizerLanes.value);

      if (componentChunks.length) {
        for (const components of componentChunks) {
          rows.push({
            type: "contentRow",
            components,
            chunkInitialId: dataIndex,
            insideSection: hasMultipleSections.value,
          });

          // We need to increase the current index by the length of the row for the next iteration.
          dataIndex += components.length;
        }
      } else {
        rows.push({
          type: "emptyRow",
          groupName,
        });
      }
    }

    // Whether or not we collapse the group, we need the footer.
    if (hasMultipleSections.value) {
      rows.push({
        type: "footer",
      });
    }
  }

  // Remove the last footer when dealing with "group by" functionality.
  if (hasMultipleSections.value) rows.pop();

  // Add filtered counter row if needed
  if (
    props.showFilteredCounter &&
    props.filteredCount !== undefined &&
    props.totalCount !== undefined
  ) {
    const hiddenCount = props.totalCount - props.filteredCount;
    if (hiddenCount > 0) {
      rows.push({
        type: "filteredCounterRow",
        hiddenCount,
      });
    }
  }

  return rows;
});

const collapseTracker = ref<Record<string, boolean>>({});
const clickCollapse = (title: string, collapsed: boolean) => {
  collapseTracker.value[title] = collapsed;
};

const componentRowsVirtualItemsList = computed(() =>
  virtualList.value.getVirtualItems(),
);

// Rows need a unique item key, so the virtualizer internal watcher knows when to recompute sizes
const getItemKey = (rowIndex: number) => {
  const row = gridRows.value[rowIndex];
  if (!row) return rowIndex;

  switch (row.type) {
    case "header":
      return `header-${row.title}`;
    case "contentRow":
      if (!hasMultipleSections.value && rowIndex === gridRows.value.length - 1)
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
  return gridRows.value.map((row, index) => {
    switch (row.type) {
      case "header":
        return GROUP_HEADER_HEIGHT;
      case "contentRow":
        if (!hasMultipleSections.value && index === gridRows.value.length - 1) {
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
  count: gridRows.value.length,
  // `virtualizerLanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
  // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
  // Our grid is based on the minimum tile width... so how many tiles can we fit?
  // thats the value of `virtualizerLanes`
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  getScrollElement: () => scrollRef.value!,
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

  for (let rowIndex = 0; rowIndex < gridRows.value.length; rowIndex++) {
    const row = gridRows.value[rowIndex];

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
    props.focusedComponentIdx === undefined ||
    props.focusedComponentIdx < 0 ||
    props.focusedComponentIdx > allVisibleComponents.value.length - 1
  )
    return;

  // otherwise use the virtualizer to scroll
  // so that even if the DOM element doesn't exist
  // it will still work!
  virtualList.value.scrollToIndex(
    getRowIndexByGridTileIndex(props.focusedComponentIdx),
    { behavior: "smooth" },
  );
};

watch([() => props.focusedComponentIdx], scrollCurrentTileIntoView);

defineEmits<{
  (e: "bulkDone"): void;
  (e: "unpin", componentId: ComponentId): void;
  (e: "childSelect", componentIdx: number): void;
  (e: "childDeselect", componentIdx: number): void;
  (e: "childHover", componentId: ComponentId): void;
  (e: "childUnhover", componentId: ComponentId): void;
  (e: "resetFilter"): void;
  (
    e: "childClicked",
    event: MouseEvent,
    componentId: ComponentId,
    componentIdx: number,
  ): void;
}>();

defineExpose({
  getGridComponentRefByIndex,
  focusedComponent,
  selectedComponents,
  allVisibleComponents,
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
