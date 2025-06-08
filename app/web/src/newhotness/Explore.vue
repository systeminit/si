<template>
  <section :class="clsx('grid h-full', showGrid ? 'explore' : 'map')">
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
          :pills="['Up', 'Down', 'Left', 'Right']"
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
              @keydown.up="() => onArrowUp()"
              @keydown.down="() => onArrowDown()"
              @keydown.esc="onEscape"
            />
          </template>
        </InstructiveVormInput>
      </div>
      <template v-if="showGrid">
        <div ref="scrollRef" class="scrollable tilegrid grow">
          <ComponentGridTile
            v-for="component in componentVirtualItemsList"
            ref="componentGridTileRefs"
            :key="filteredComponents[component.index]!.id"
            :class="clsx(tileClasses(component.index))"
            :component="filteredComponents[component.index]!"
            @mouseenter="hover(component.index)"
            @mouseleave="unhover(component.index)"
            @click.stop.left="(e) => componentClicked(e, filteredComponents[component.index]!.id)"
            @click.stop.right="(e) => componentClicked(e, filteredComponents[component.index]!.id)"
          />
          <div
            v-if="
              componentList.length === 0 && componentListRaw.isSuccess.value
            "
          >
            <em>No components in View</em>
          </div>
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
            pill="Cmd + A"
            tone="action"
            size="sm"
            @click="openAddComponentModal"
          />
        </footer>
      </template>
      <Map v-else :active="!showGrid" />
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
      :componentIds="
        interactionTargetComponentId ? [interactionTargetComponentId] : []
      "
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
  BifrostComponent,
  BifrostComponentList,
  BifrostViewList,
  ViewComponentList,
  EntityKind,
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
import { KeyDetails, keyEmitter } from "./logic_composables/emitters";
import TabGroupToggle from "./layout_components/TabGroupToggle.vue";
import { SelectionsInQueryString } from "./Workspace.vue";
import AddComponentModal from "./AddComponentModal.vue";
import AddViewModal from "./AddViewModal.vue";
import ComponentContextMenu from "./ComponentContextMenu.vue";

const router = useRouter();
const route = useRoute();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const selectedView = ref("");
const groupRef = ref<InstanceType<typeof TabGroupToggle>>();
const actionsRef = ref<typeof CollapsingGridItem>();
const historyRef = ref<typeof CollapsingGridItem>();
const componentGridTileRefs = ref<InstanceType<typeof ComponentGridTile>[]>();

const getGridTileIndexByComponentId = (id: ComponentId) => {
  return componentVirtualItemsList.value.findIndex(
    (item) => filteredComponents[item.index]!.id === id,
  );
};

const urlGridOrMap = computed(() => {
  const q: SelectionsInQueryString = router.currentRoute.value?.query;
  const keys = Object.keys(q);
  if (keys.includes("grid")) return "grid";
  if (keys.includes("map")) return "map";
  return "grid";
});
const showGrid = computed(() => groupRef.value?.isA);
watch(showGrid, () => {
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

const scrollRef = ref<HTMLDivElement>();

const filteredComponents = reactive<BifrostComponent[]>([]);

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

const lanes = computed(() =>
  // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
  Math.floor((scrollRef.value?.offsetWidth ?? 0) / 250),
);

const virtualizerOptions = computed(() => {
  return {
    count: filteredComponents.length,
    // `lanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
    // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
    // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
    // thats the value of `lanes`
    lanes: lanes.value,
    getScrollElement: () => scrollRef.value!,
    estimateSize: () => 200,
    overscan: 3,
  };
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
  else if (selected) return ""; // TODO(WENDY) - not using selected yet!
  else return "";
};
const hoverByComponentId = (id: ComponentId) => {
  const index = getGridTileIndexByComponentId(id);

  if (index !== -1) selectorGridPosition.value = index;
};
const hover = (index: number) => {
  // if (focused.value) return; // No hovering while focused!
  selectorGridPosition.value = index;
};
const unhover = (index?: number) => {
  // if (focused.value) return; // unhover only if not focused!
  if (!index || selectorGridPosition.value === index) {
    selectorGridPosition.value = -1;
    // unfocus();
  }
};

const focus = (componentId: ComponentId) => {
  if (!componentGridTileRefs.value) return;
  hoverByComponentId(componentId);
  focusedComponentId.value = componentId;
  focusGridPosition.value = selectorGridPosition.value;
  const gridTile =
    componentGridTileRefs.value[getGridTileIndexByComponentId(componentId)]!;
  componentContextMenuRef.value?.open(gridTile, [componentId]);
};
const unfocus = () => {
  focusedComponentId.value = undefined;

  selectorGridPosition.value = focusGridPosition.value;
  focusGridPosition.value = -1;
  componentContextMenuRef.value?.close();
};

const searchRef = ref<InstanceType<typeof VormInput>>();
const mountKeyEmitters = () => {
  keyEmitter.on("k", onK);
  keyEmitter.on("a", onA);
  keyEmitter.on("e", onE);
  keyEmitter.on("d", onD);
  keyEmitter.on("ArrowDown", onArrowDown);
  keyEmitter.on("ArrowUp", onArrowUp);
  keyEmitter.on("ArrowLeft", onArrowLeft);
  keyEmitter.on("ArrowRight", onArrowRight);
  keyEmitter.on("Enter", onEnter);
  keyEmitter.on("Tab", onTab);
  keyEmitter.on("Escape", onEscape);
};
const removeKeyEmitters = () => {
  keyEmitter.off("k", onK);
  keyEmitter.off("a", onA);
  keyEmitter.off("e", onE);
  keyEmitter.off("d", onD);
  keyEmitter.off("ArrowDown", onArrowDown);
  keyEmitter.off("ArrowUp", onArrowUp);
  keyEmitter.off("ArrowLeft", onArrowLeft);
  keyEmitter.off("ArrowRight", onArrowRight);
  keyEmitter.off("Enter", onEnter);
  keyEmitter.off("Tab", onTab);
  keyEmitter.off("Escape", onEscape);
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
};

const onK = (e: KeyDetails["k"]) => {
  if (e.metaKey || e.ctrlKey) {
    searchRef.value?.focus();
  }
};
const onA = (e: KeyDetails["a"]) => {
  if (e.metaKey || e.ctrlKey) {
    openAddComponentModal();
  }
};
const onE = (e: KeyDetails["e"]) => {
  if (selectorGridPosition.value !== -1 && (e.metaKey || e.ctrlKey)) {
    if (!focusedComponentId.value) return;
    componentContextMenuRef.value?.componentsStartErase([
      focusedComponentId.value,
    ]);
  }
};
const onD = (e: KeyDetails["e"]) => {
  e.preventDefault();

  if (e.metaKey || e.ctrlKey) {
    if (!interactionTargetComponentId.value) return;
    componentContextMenuRef.value?.componentDuplicate();
  }
};
const onArrowUp = () => {
  if (!showGrid.value) return;
  if (focusedComponentId.value) unfocus();
  selectorGridPosition.value -= lanes.value;
  constrainPosition();
};
const onArrowDown = () => {
  if (!showGrid.value) return;
  if (focusedComponentId.value) unfocus();
  selectorGridPosition.value += lanes.value;
  constrainPosition();
};
const onArrowLeft = () => {
  previousComponent();
};
const onArrowRight = () => {
  nextComponent();
};
const onEscape = () => {
  searchRef.value?.blur();
  selectorGridPosition.value = -1;
};
const onTab = (
  e: { preventDefault: () => void; shiftKey: boolean },
  blurSearch = false,
) => {
  e.preventDefault();
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
const onEnter = () => {
  if (selectorGridPosition.value !== -1) {
    const componentId = filteredComponents[selectorGridPosition.value]?.id;
    if (!componentId) return;
    componentInteract(componentId);
  }
};
const onClick = (_e: MouseEvent) => {
  // general click handler for the whole page
  // any click which doesn't do this behavior should have .stop on it!
  unfocus();
  unhover();
};
onMounted(() => {
  removeKeyEmitters();
  mountKeyEmitters();
  document.addEventListener("click", onClick);
});
onBeforeUnmount(() => {
  removeKeyEmitters();
  document.removeEventListener("click", onClick);
});

const componentClicked = (e: MouseEvent, componentId: ComponentId) => {
  e.preventDefault();
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

const componentInteract = (componentId: ComponentId) => {
  if (focusedComponentId.value) {
    componentNavigate(componentId);
  } else {
    focus(componentId);
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
