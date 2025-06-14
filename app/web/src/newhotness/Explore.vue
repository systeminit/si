<template>
  <DelayedLoader v-if="componentListRaw.isLoading.value" :size="'full'" />
  <section v-else :class="clsx('grid h-full', showGrid ? 'explore' : 'map')">
    <!-- Left column -->
    <!-- 12 pixel padding to align with the SI logo -->
    <div
      class="main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]"
    >
      <div class="flex-none flex flex-row items-center gap-xs">
        <TabGroupToggle ref="groupRef" :aOrB="urlGridOrMap === 'grid'">
          <template #a="{ selected, toggle }">
            <VButton
              label="Grid"
              size="sm"
              variant="ghost"
              :tone="selected ? 'action' : 'shade'"
              @click.stop="toggle"
            />
          </template>
          <template #b="{ selected, toggle }">
            <VButton
              label="Map"
              size="sm"
              variant="ghost"
              :tone="selected ? 'action' : 'shade'"
              @click.stop="toggle"
            />
          </template>
        </TabGroupToggle>
        <DropdownMenuButton
          class="rounded min-w-[128px]"
          :options="viewListOptions"
          :modelValue="selectedView"
          placeholder="All Views"
          @update:modelValue="(val) => (selectedView = val)"
        >
          <template #afterOptions>
            <DropdownMenuItem
              label="Add a View"
              icon="plus"
              @select="openViewModal"
            />
          </template>
        </DropdownMenuButton>
        <InstructiveVormInput
          :class="clsx('rounded grow cursor-text')"
          :activeClasses="
            themeClasses('border-action-500', 'border-action-300')
          "
          :inactiveClasses="
            themeClasses(
              'border-neutral-400 hover:border-black',
              'border-neutral-600 hover:border-white',
            )
          "
          :pills="
            CONTROL_SCHEME === 'v1' ? ['Up', 'Down', 'Left', 'Right'] : ['Tab']
          "
          instructions="to navigate"
          @click="searchRef?.focus()"
        >
          <template #left>
            <Icon name="search" tone="neutral" size="sm" />
          </template>
          <template #default="slotProps">
            <VormInput
              ref="searchRef"
              v-model="searchString"
              autocomplete="off"
              :class="slotProps.class"
              noStyles
              placeholder="Search components"
              @focus="slotProps.focus"
              @blur="slotProps.blur"
              @keydown.tab="(e: KeyboardEvent) => onTab(e, true)"
              @keydown.left="() => previousComponent()"
              @keydown.right="() => nextComponent()"
              @keydown.up="onArrowUp"
              @keydown.down="onArrowDown"
              @keydown.esc="onEscape"
            />
          </template>
        </InstructiveVormInput>
      </div>
      <template v-if="showGrid">
        <div
          ref="scrollRef"
          class="scrollable tilegrid grow"
          @scroll="onScroll"
          @scrollend="fixContextMenuAfterScroll"
        >
          <ComponentGridTile
            v-for="(component, index) in componentVirtualItemsList"
            ref="componentGridTileRefs"
            :key="filteredComponents[component.index]!.id"
            :data-index="index"
            :class="clsx(tileClasses(component.index))"
            :component="filteredComponents[component.index]!"
            @mouseenter="hover(component.index)"
            @mouseleave="unhover(component.index)"
            @click.stop.left="
              (e) =>
                componentClicked(e, filteredComponents[component.index]!.id)
            "
            @click.stop.right="
              (e) =>
                componentClicked(e, filteredComponents[component.index]!.id)
            "
          />
          <EmptyState
            v-if="
              componentList.length === 0 && componentListRaw.isSuccess.value
            "
            icon="component"
            text="No components in view"
          />
        </div>
        <footer
          :class="
            clsx(
              'flex-none h-12 p-2xs border-t flex flex-row justify-end items-center',
              themeClasses(
                'bg-neutral-100 border-neutral-400',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
        >
          <!-- footer -->
          <VButton
            label="Add a component"
            pill="N"
            tone="action"
            size="sm"
            @click="openAddComponentModal"
          />
        </footer>
      </template>
      <Map v-else ref="mapRef" :active="!showGrid" />
    </div>
    <!-- Right column -->
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
      <div class="grow grid grid-rows-subgrid" :style="collapsingStyles">
        <CollapsingGridItem ref="actionsRef">
          <template #header>Actions ({{ actionViewList.length }})</template>
          <ul class="actions list">
            <ActionCard
              v-for="action in actionViewList"
              :key="action.id"
              :action="action"
              :selected="false"
              :noInteraction="false"
            />
          </ul>
        </CollapsingGridItem>
        <CollapsingGridItem ref="historyRef" disableScroll>
          <template #header>History</template>
          <FuncRunList :limit="25" />
        </CollapsingGridItem>
      </div>
      <div
        :class="
          clsx(
            'flex-none h-12 border-t flex flex-col justify-between p-2xs',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <Breadcrumbs class="text-xs" />
        <RealtimeStatusPageState />
      </div>
    </div>
    <AddComponentModal ref="addComponentModalRef" />
    <AddViewModal
      ref="addViewModalRef"
      :views="viewListQuery.data.value?.views"
    />
    <ComponentContextMenu
      ref="componentContextMenuRef"
      onGrid
      :enableKeyboardControls="CONTROL_SCHEME === 'v2'"
      @edit="openFocusedComponent"
    />
  </section>
</template>

<script lang="ts" setup>
// TODO(Wendy) - we should clean up these non-null assertions!
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import {
  computed,
  inject,
  onBeforeUnmount,
  onMounted,
  provide,
  reactive,
  ref,
  watch,
} from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  themeClasses,
  VormInput,
  VButton,
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { Fzf } from "fzf";
import { tw } from "@si/vue-lib";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  BifrostComponentList,
  BifrostViewList,
  ViewComponentList,
  EntityKind,
  BifrostComponentInList,
} from "@/workers/types/entity_kind_types";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import Map from "./Map.vue";
import { collapsingGridStyles } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import ComponentGridTile from "./ComponentGridTile.vue";
import Breadcrumbs from "./layout_components/Breadcrumbs.vue";
import ActionCard from "./ActionCard.vue";
import FuncRunList from "./FuncRunList.vue";
import { assertIsDefined, Context, ExploreContext } from "./types";
import DelayedLoader from "./layout_components/DelayedLoader.vue";
import {
  KeyDetails,
  keyEmitter,
  windowResizeEmitter,
  windowWidthReactive,
} from "./logic_composables/emitters";
import TabGroupToggle from "./layout_components/TabGroupToggle.vue";
import { SelectionsInQueryString } from "./Workspace.vue";
import AddComponentModal from "./AddComponentModal.vue";
import AddViewModal from "./AddViewModal.vue";
import ComponentContextMenu from "./ComponentContextMenu.vue";
import EmptyState from "./EmptyState.vue";
import { elementIsScrolledIntoView } from "./logic_composables/dom_funcs";

type ControlScheme = "v1" | "v2";
const CONTROL_SCHEME: ControlScheme = "v2" as ControlScheme;

const router = useRouter();
const route = useRoute();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const selectedView = ref("");
const groupRef = ref<InstanceType<typeof TabGroupToggle>>();
const actionsRef = ref<typeof CollapsingGridItem>();
const historyRef = ref<typeof CollapsingGridItem>();
const mapRef = ref<InstanceType<typeof Map>>();
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

const getGridTileIndexByComponentId = (id: ComponentId) => {
  return componentVirtualItemsList.value.findIndex(
    (item) => filteredComponents[item.index]!.id === id,
  );
};
const getGridTileByIndex = (idx: number) => {
  if (componentGridTileRefs.value) {
    const tile = componentGridTileRefs.value.find((t) => {
      return Number(t.$el.dataset.index) === idx;
    });
    return tile;
  }
  return undefined;
};

const urlGridOrMap = computed(() => {
  const q: SelectionsInQueryString = router.currentRoute.value?.query;
  const keys = Object.keys(q);
  if (keys.includes("grid")) return "grid";
  if (keys.includes("map")) return "map";
  return "grid";
});
const showGrid = computed(() => (groupRef.value ? groupRef.value.isA : true));
watch(showGrid, () => {
  unfocus();
  unhover();
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.grid;
  if (showGrid.value) query.grid = "1";
  else query.map = "1";
  router.push({ query });
});

const key = useMakeKey();
const args = useMakeArgs();

const viewListQuery = useQuery<BifrostViewList | null>({
  queryKey: key(EntityKind.ViewList),
  queryFn: async () =>
    await bifrost<BifrostViewList>(args(EntityKind.ViewList)),
});
const viewListOptions = computed(() => {
  const list = viewListQuery.data.value?.views || [];
  const options = [{ value: "", label: "All Views" }];
  return options.concat(
    list.map((l) => {
      return { value: l.id, label: l.name };
    }),
  );
});

const selectedViewOrDefaultId = computed(() => {
  if (selectedView.value) return selectedView.value;
  if (!viewListQuery.data.value) return "";
  const view = viewListQuery.data.value.views.find((v) => v.isDefault);
  if (!view) return "";
  return view.id;
});

const exploreContext = computed<ExploreContext>(() => {
  return {
    viewId: selectedViewOrDefaultId,
  };
});

provide("EXPLORE_CONTEXT", exploreContext.value);

const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

const kind = computed(() =>
  selectedView.value ? EntityKind.ViewComponentList : EntityKind.ComponentList,
);
const id = computed(() =>
  selectedView.value ? selectedView.value : ctx.workspacePk.value,
);
const componentQueryKey = key(kind, id);

const componentListRaw = useQuery<
  BifrostComponentList | ViewComponentList | null
>({
  queryKey: componentQueryKey,
  queryFn: async () => {
    const arg = selectedView.value
      ? args(EntityKind.ViewComponentList, selectedView.value)
      : args(EntityKind.ComponentList);
    return await bifrost<BifrostComponentList | ViewComponentList>(arg);
  },
});

const componentList = computed(
  () => componentListRaw.data.value?.components ?? [],
);
const componentsById = computed(() =>
  Object.fromEntries(componentList.value.map((c) => [c.id, c])),
);

const scrollRef = ref<HTMLDivElement>();

const filteredComponents = reactive<BifrostComponentInList[]>([]);

const searchString = ref("");
const computedSearchString = computed(() => searchString.value);

// send this down to any components that might use it
provide("SEARCH", computedSearchString);

watch(
  () => [searchString.value, componentList.value],
  () => {
    if (!searchString.value) {
      filteredComponents.splice(0, Infinity, ...componentList.value);
      return;
    }

    const fzf = new Fzf(componentList.value, {
      casing: "case-insensitive",
      selector: (c) =>
        `${c.name} ${c.schemaVariantName} ${c.schemaName} ${c.schemaCategory} ${c.schemaId} ${c.id}`,
    });

    const results = fzf.find(searchString.value);
    filteredComponents.splice(0, Infinity, ...results.map((fz) => fz.item));
  },
  { immediate: true },
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

// This computes the expected number of components in a row based on the width of the scroll area
const virtualizerLanes = computed(() => {
  // We need to force a recompute of this value when the screen is resized
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  windowWidthReactive.value;

  // We also need to force a recompute of this value if the number of tiles changes
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  componentGridTileRefs.value;

  // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
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
    availableSpace -= 250; // width of one grid tile
    if (availableSpace > 0) {
      newLanes++;
    }
    availableSpace -= 16; // gap between grid tiles
  }
  return newLanes;
});

const virtualizerOptions = computed(() => {
  const options = {
    count: filteredComponents.length,
    // `virtualizerLanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
    // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
    // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
    // thats the value of `virtualizerLanes`
    lanes: virtualizerLanes.value,
    getScrollElement: () => scrollRef.value!,
    estimateSize: () => 200,
    overscan: 3,
  };
  return options;
});

const virtualList = useVirtualizer(virtualizerOptions);

const componentVirtualItemsList = computed(() =>
  virtualList.value.getVirtualItems(),
);

const collapsingStyles = computed(() =>
  collapsingGridStyles([
    actionsRef.value?.openState,
    historyRef.value?.openState,
  ]),
);

const selectedComponentIds = reactive<Set<string>>(new Set());
const selectorGridPosition = ref<number>(-1);
const focusedComponentId = ref<string | undefined>();
const hoveredComponentId = computed(
  () => filteredComponents[selectorGridPosition.value]?.id,
);
const interactionTargetComponentId = computed(
  () => focusedComponentId.value ?? hoveredComponentId.value,
);
const focusGridPosition = ref<number>(-1);
const constrainPosition = () => {
  selectorGridPosition.value = Math.min(
    filteredComponents.length - 1,
    Math.max(-1, selectorGridPosition.value),
  );
  scrollCurrentTileIntoView();
};
const isSelected = (idx: number) =>
  selectedComponentIds.has(filteredComponents[idx]!.id);
const isHovered = (idx: number) => selectorGridPosition.value === idx;
const isFocused = (idx: number) =>
  focusGridPosition.value === idx && focusedComponentId.value;
const tileClasses = (idx: number) => {
  const selected = isSelected(idx);
  const hovered = isHovered(idx);
  const focused = isFocused(idx);
  if (focused)
    return themeClasses(tw`border-action-500`, tw`border-action-300`);
  else if (hovered) return themeClasses(tw`border-black`, tw`border-white`);
  // TODO(WENDY) - not using selected yet!
  else if (selected) return "";
  else return "";
};
const hoverByComponentId = (id: ComponentId) => {
  const index = getGridTileIndexByComponentId(id);

  if (index !== -1) selectorGridPosition.value = index;
};
const hover = (index: number) => {
  selectorGridPosition.value = index;
};
const unhover = (index?: number) => {
  if (!index || selectorGridPosition.value === index) {
    selectorGridPosition.value = -1;
  }
};

const focus = (componentId: ComponentId) => {
  if (!componentGridTileRefs.value) return;
  hoverByComponentId(componentId);
  focusedComponentId.value = componentId;
  focusGridPosition.value = selectorGridPosition.value;
  const gridTileIndex = getGridTileIndexByComponentId(componentId);
  const gridTile = getGridTileByIndex(gridTileIndex);
  if (gridTile) {
    const component = componentsById.value[componentId];
    if (component) {
      componentContextMenuRef.value?.open(gridTile, [component]);
    }
  }
};
const unfocus = () => {
  focusedComponentId.value = undefined;

  selectorGridPosition.value = focusGridPosition.value;
  focusGridPosition.value = -1;
  componentContextMenuRef.value?.close();
};

const searchRef = ref<InstanceType<typeof VormInput>>();
const mountEmitters = () => {
  removeEmitters();
  keyEmitter.on("k", onK);
  keyEmitter.on("n", onN);
  keyEmitter.on("e", onE);
  keyEmitter.on("d", onD);
  keyEmitter.on("u", onU);
  keyEmitter.on("r", onR);
  keyEmitter.on("ArrowDown", onArrowDown);
  keyEmitter.on("ArrowUp", onArrowUp);
  keyEmitter.on("ArrowLeft", onArrowLeft);
  keyEmitter.on("ArrowRight", onArrowRight);
  keyEmitter.on("Enter", onEnter);
  keyEmitter.on("Tab", onTab);
  keyEmitter.on("Escape", onEscape);
  keyEmitter.on("Backspace", onBackspace);
  windowResizeEmitter.on("resize", onResize);
};
const removeEmitters = () => {
  keyEmitter.off("k", onK);
  keyEmitter.off("n", onN);
  keyEmitter.off("e", onE);
  keyEmitter.off("d", onD);
  keyEmitter.off("u", onU);
  keyEmitter.off("r", onR);
  keyEmitter.off("ArrowDown", onArrowDown);
  keyEmitter.off("ArrowUp", onArrowUp);
  keyEmitter.off("ArrowLeft", onArrowLeft);
  keyEmitter.off("ArrowRight", onArrowRight);
  keyEmitter.off("Enter", onEnter);
  keyEmitter.off("Tab", onTab);
  keyEmitter.off("Escape", onEscape);
  keyEmitter.off("Backspace", onBackspace);
  windowResizeEmitter.off("resize", onResize);
};
const nextComponent = (wrap = false) => {
  if (!showGrid.value) return;
  if (focusedComponentId.value) unfocus();
  selectorGridPosition.value += 1;
  if (wrap && selectorGridPosition.value > filteredComponents.length - 1) {
    selectorGridPosition.value = -1;
    searchRef.value?.focus();
  }
  constrainPosition();
  if (CONTROL_SCHEME === "v2" && hoveredComponentId.value) {
    focus(hoveredComponentId.value);
  }
};
const previousComponent = (wrap = false) => {
  if (!showGrid.value) return;
  if (focusedComponentId.value) unfocus();
  selectorGridPosition.value -= 1;
  if (wrap) {
    if (selectorGridPosition.value < -1) {
      selectorGridPosition.value = filteredComponents.length - 1;
    } else if (selectorGridPosition.value < 0) {
      selectorGridPosition.value = -1;
      searchRef.value?.focus();
    }
  }
  constrainPosition();
  if (CONTROL_SCHEME === "v2" && hoveredComponentId.value) {
    focus(hoveredComponentId.value);
  }
};

const onK = (e: KeyDetails["k"]) => {
  e.preventDefault();

  // same behavior on the grid and map!
  searchRef.value?.focus();
};
const onN = (e: KeyDetails["n"]) => {
  e.preventDefault();

  // same behavior on the grid and map!
  openAddComponentModal();
};
const onE = (e: KeyDetails["e"]) => {
  e.preventDefault();
  if (showGrid.value) {
    if (selectorGridPosition.value !== -1) {
      if (!interactionTargetComponentId.value) return;
      componentContextMenuRef.value?.componentsStartErase([
        interactionTargetComponentId.value,
      ]);
    }
  } else {
    mapRef.value?.onE(e);
  }
};
const onD = (e: KeyDetails["d"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (e.metaKey || e.ctrlKey) {
      if (!interactionTargetComponentId.value) return;
      componentContextMenuRef.value?.componentDuplicate([
        interactionTargetComponentId.value,
      ]);
    }
  } else {
    mapRef.value?.onD(e);
  }
};
const onU = (e: KeyDetails["u"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!interactionTargetComponentId.value) return;
    const targetComponent = filteredComponents.find(
      (comp) => comp.id === interactionTargetComponentId.value,
    );
    if (targetComponent && targetComponent.canBeUpgraded) {
      componentContextMenuRef.value?.componentUpgrade([
        interactionTargetComponentId.value,
      ]);
    }
  } else {
    mapRef.value?.onU(e);
  }
};
const onBackspace = (e: KeyDetails["Backspace"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!interactionTargetComponentId.value) return;
    const component = componentsById.value[interactionTargetComponentId.value];
    if (!component) return;
    componentContextMenuRef.value?.componentsStartDelete([component]);
  } else {
    mapRef.value?.onBackspace(e);
  }
};
const onR = (e: KeyDetails["r"]) => {
  if (e.metaKey || e.ctrlKey) {
    // This is the chrome hotkey combo for refreshing the page! Let it happen!
    return;
  }
  e.preventDefault();

  if (showGrid.value) {
    if (!interactionTargetComponentId.value) return;
    const targetComponent = filteredComponents.find(
      (comp) => comp.id === interactionTargetComponentId.value,
    );
    if (targetComponent && targetComponent.canBeUpgraded) {
      componentContextMenuRef.value?.componentsRestore([
        interactionTargetComponentId.value,
      ]);
    }
  } else {
    mapRef.value?.onR(e);
  }
};
const onArrowUp = (e: KeyDetails["ArrowUp"]) => {
  if (CONTROL_SCHEME === "v2" && showGrid.value) return;
  e.preventDefault();
  if (showGrid.value) {
    if (focusedComponentId.value) unfocus();
    selectorGridPosition.value -= lanes.value;
    constrainPosition();
  } else {
    mapRef.value?.onArrowUp();
  }
};
const onArrowDown = (e: KeyDetails["ArrowDown"]) => {
  if (CONTROL_SCHEME === "v2" && showGrid.value) return;
  e.preventDefault();
  if (showGrid.value) {
    if (focusedComponentId.value) unfocus();
    if (selectorGridPosition.value === -1) {
      selectorGridPosition.value = 0;
    } else {
      selectorGridPosition.value += lanes.value;
    }
    constrainPosition();
  } else {
    mapRef.value?.onArrowDown();
  }
};
const onArrowLeft = () => {
  if (CONTROL_SCHEME === "v2" && showGrid.value) return;
  if (showGrid.value) {
    previousComponent();
  } else {
    mapRef.value?.onArrowLeft();
  }
};
const onArrowRight = () => {
  if (CONTROL_SCHEME === "v2" && showGrid.value) return;
  if (showGrid.value) {
    nextComponent();
  } else {
    mapRef.value?.onArrowRight();
  }
};
const onEscape = () => {
  if (showGrid.value) {
    searchRef.value?.blur();
    selectorGridPosition.value = -1;
  } else {
    mapRef.value?.onEscape();
  }
};
const onTab = (e: KeyDetails["Tab"], blurSearch = false) => {
  e.preventDefault();
  if (!showGrid.value) return; // no tab behavior on the map yet

  const pageFunc = e.shiftKey ? previousComponent : nextComponent;
  if (!searchRef.value) return;
  else if (blurSearch) {
    searchRef.value.blur();
    pageFunc(true);
  } else if (selectorGridPosition.value === -1 && !searchRef.value.isFocus) {
    searchRef.value.focus();
  } else {
    pageFunc(true);
  }
};
const onEnter = (e: KeyDetails["Enter"]) => {
  if (CONTROL_SCHEME === "v2" && focusedComponentId.value) {
    // enter controls the context menu, not the grid tile
    return;
  }
  e.preventDefault();
  if (!showGrid.value) return; // no enter behavior on the map yet

  if (selectorGridPosition.value !== -1) {
    const componentId = filteredComponents[selectorGridPosition.value]?.id;
    if (!componentId) return;
    componentInteract(componentId);
  }
};
const onScroll = () => {
  if (focusedComponentId.value && CONTROL_SCHEME === "v1") {
    unfocus();
  } else {
    // for the v2 interface, close the menu while scrolling
    componentContextMenuRef.value?.close();
  }
};
const fixContextMenuAfterScroll = () => {
  // For the v2 control scheme, we need to fix the context menu after scrolling
  // If the element is scrolled into view, show the menu
  // If the element is scrolled offscreen, unfocus/unhover as per v1
  if (CONTROL_SCHEME === "v1") return;
  else if (focusedComponentId.value) {
    const tileIndex = getGridTileIndexByComponentId(focusedComponentId.value);
    const tile = getGridTileByIndex(tileIndex);
    const el = tile?.$el;
    if (el && elementIsScrolledIntoView(el)) {
      focus(focusedComponentId.value);
    } else {
      unfocus();
      unhover();
    }
  }
};
const onResize = () => {
  unfocus();
  unhover();
};
const onClick = (e: MouseEvent) => {
  const inside =
    componentContextMenuRef.value?.contextMenuRef?.elementIsInsideMenu;
  if (inside && e.target instanceof Node && inside(e.target)) {
    return;
  }

  // general click handler for the whole page
  // any click which doesn't do this behavior should have .stop on it!
  unfocus();
  unhover();
};
onMounted(() => {
  mountEmitters();
  document.addEventListener("click", onClick);
});
onBeforeUnmount(() => {
  removeEmitters();
  document.removeEventListener("click", onClick);
});

const componentClicked = (e: MouseEvent, componentId: ComponentId) => {
  e.preventDefault();
  if (CONTROL_SCHEME === "v1") {
    componentClickedV1(e, componentId);
  } else {
    componentClickedV2(e, componentId);
  }
};
const componentClickedV1 = (e: MouseEvent, componentId: ComponentId) => {
  if (
    focusedComponentId.value &&
    selectorGridPosition.value !== focusGridPosition.value
  ) {
    unfocus();
    focus(componentId);
  } else {
    hoverByComponentId(componentId); // should already be hovered but let's make sure!
    componentInteract(componentId);
  }
};
const componentClickedV2 = (e: MouseEvent, componentId: ComponentId) => {
  if (e.button === 0) {
    componentNavigate(componentId);
  } else {
    componentClickedV1(e, componentId);
  }
};

const componentInteract = (componentId: ComponentId) => {
  if (focusedComponentId.value) {
    componentNavigate(componentId);
  } else {
    focus(componentId);
  }
};

const openFocusedComponent = () => {
  if (focusedComponentId.value) {
    componentNavigate(focusedComponentId.value);
  }
};

const componentNavigate = (componentId: ComponentId) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
  });
};

const addComponentModalRef = ref<InstanceType<typeof AddComponentModal>>();

const openAddComponentModal = () => {
  addComponentModalRef.value?.open();
};

const addViewModalRef = ref<InstanceType<typeof AddViewModal>>();

const openViewModal = () => {
  addViewModalRef.value?.open();
};

const componentContextMenuRef =
  ref<InstanceType<typeof ComponentContextMenu>>();

const scrollCurrentTileIntoView = () => {
  const tile = getGridTileByIndex(selectorGridPosition.value);
  tile?.$el.scrollIntoView({ behavior: "smooth", block: "nearest" });
};
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
