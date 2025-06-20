<template>
  <DelayedLoader v-if="componentListRaw.isLoading.value" :size="'full'" />
  <section v-else :class="clsx('grid h-full', showGrid ? 'explore' : 'map')">
    <!-- Left column -->
    <!-- 12 pixel padding to align with the SI logo -->
    <div
      class="main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]"
    >
      <div class="flex-none flex flex-row items-center gap-xs">
        <DropdownMenuButton
          class="rounded min-w-[128px]"
          :options="viewListOptions"
          :modelValue="selectedView"
          minWidthToAnchor
          placeholder="All Views"
          checkable
          enableSettingsEdit
          @edit="openEditViewModal"
          @update:modelValue="(val) => (selectedView = val)"
        >
          <template #beforeOptions>
            <DropdownMenuItem
              label="All Views"
              value="''"
              checkable
              :checked="selectedView === ''"
              @select="() => (selectedView = '')"
            />
          </template>
          <template #afterOptions>
            <DropdownMenuItem
              class="border-t"
              label="Add a View"
              icon="plus"
              disableCheckable
              @select="openAddViewModal"
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
            showGrid
              ? CONTROL_SCHEME === 'v1'
                ? ['Up', 'Down', 'Left', 'Right']
                : ['Tab']
              : undefined
          "
          :instructions="showGrid ? 'to navigate' : undefined"
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
              @focus="
                () => {
                  slotProps.focus();
                  mapRef?.deselect();
                  // FIXME(nick,victor): this needs to work!
                  // unfocus();
                  // unhover();
                }
              "
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
      <div class="flex-none flex flex-row items-center gap-xs justify-between">
        <TabGroupToggle
          ref="groupRef"
          :aOrB="urlGridOrMap === 'grid'"
          @toggle="storeViewMode"
        >
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
        <div>
          <DropdownMenuButton
            :class="clsx('rounded')"
            :options="groupByOptions"
            :modelValue="groupBySelection"
            placeholder="Group by"
            minWidthToAnchor
            checkable
            alwaysShowPlaceholder
            highlightWhenModelValue
            @update:modelValue="(val) => (groupBySelection = val)"
          >
            <template #beforeOptions>
              <DropdownMenuItem
                label="None"
                value="''"
                checkable
                :checked="groupBySelection === ''"
                @select="() => (groupBySelection = '')"
              />
            </template>
          </DropdownMenuButton>
        </div>
      </div>
      <template v-if="showGrid">
        <div
          v-if="componentList.length === 0 && componentListRaw.isSuccess.value"
          class="flex-1 overflow-hidden flex flex-row items-center justify-center"
        >
          <EmptyState icon="component" text="No components in view" />
        </div>
        <div
          v-else-if="groupBySelection"
          ref="scrollRef"
          class="scrollable grow"
          style="overflow-anchor: none"
          @scroll="onScroll"
          @scrollend="fixContextMenuAfterScroll"
        >
          <ExploreComponentGrid
            v-for="(components, title) in groupedComponents"
            :key="title"
            :title="title"
            :components="components"
            :scrollRef="scrollRef"
          />
        </div>
        <div
          v-else
          ref="scrollRef"
          class="scrollable grow"
          style="overflow-anchor: none"
          @scroll="onScroll"
          @scrollend="fixContextMenuAfterScroll"
        >
          <ExploreComponentGrid
            :components="filteredComponents"
            :scrollRef="scrollRef"
          />
        </div>
        <footer
          :class="
            clsx(
              'flex-none h-12 px-xs border-t flex flex-row items-center',
              'justify-between',
              themeClasses(
                'bg-neutral-100 border-neutral-400',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
        >
          <!-- footer -->
          <VButton
            label="See keyboard shortcuts"
            pill="?"
            tone="neutral"
            size="sm"
            @click="openShortcutModal"
          />
          <VButton
            label="Add a component"
            pill="N"
            tone="action"
            size="sm"
            @click="openAddComponentModal"
          />
        </footer>
      </template>
      <Map
        v-else
        ref="mapRef"
        :active="!showGrid"
        :components="componentList"
        @deselect="onMapDeselect"
        @help="openShortcutModal"
      />
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
        <RealtimeStatusPageState />
      </div>
    </div>

    <!-- MODALS -->
    <ShortcutModal ref="shortcutModalRef" />
    <AddComponentModal ref="addComponentModalRef" />
    <AddViewModal
      ref="addViewModalRef"
      :views="viewListQuery.data.value ?? []"
    />
    <!-- For the edit view modals, upon delete, change back to "All Views" -->
    <EditViewModal
      ref="editViewModalRef"
      @deleted="() => (selectedView = '')"
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
import * as _ from "lodash-es";
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
import { Fzf } from "fzf";
import {
  bifrost,
  bifrostList,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  EntityKind,
  ComponentInList,
  View,
} from "@/workers/types/entity_kind_types";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import { Listable } from "@/workers/types/dbinterface";
import Map from "./Map.vue";
import { collapsingGridStyles } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import ComponentGridTile, {
  getQualificationSummary,
  GRID_TILE_HEIGHT,
} from "./ComponentGridTile.vue";
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
import EditViewModal from "./EditViewModal.vue";
import ComponentContextMenu from "./ComponentContextMenu.vue";
import EmptyState from "./EmptyState.vue";
import { elementIsScrolledIntoView } from "./logic_composables/dom_funcs";
import ShortcutModal from "./ShortcutModal.vue";
import { useUpgrade } from "./logic_composables/upgrade";
import ExploreComponentGrid from "./ExploreComponentGrid.vue";

// MAKE SURE THESE NUMBERS STAY ACCURATE IF YOU CHANGE THE GRID!
const MIN_GRID_TILE_WIDTH = 250;
const GRID_TILE_GAP = 16;

type ControlScheme = "v1" | "v2";
const CONTROL_SCHEME: ControlScheme = "v2" as ControlScheme;

const router = useRouter();
const route = useRoute();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const VIEW_MODE_LOCAL_STORAGE_KEY = "newhotness-view-mode";
const viewModeStorageKey = () =>
  `${VIEW_MODE_LOCAL_STORAGE_KEY}: ${ctx.changeSetId}`;

const selectedView = ref("");
const groupRef = ref<InstanceType<typeof TabGroupToggle>>();
const actionsRef = ref<typeof CollapsingGridItem>();
const historyRef = ref<typeof CollapsingGridItem>();
const mapRef = ref<InstanceType<typeof Map>>();

const getGridTileIndexByComponentId = (id: ComponentId) => {
  return filteredComponents.findIndex((component) => component.id === id);
};

const urlGridOrMap = computed(() => {
  const q: SelectionsInQueryString = router.currentRoute.value?.query;
  const keys = Object.keys(q);
  if (keys.includes("grid")) return "grid";
  if (keys.includes("map")) return "map";
  const mode = localStorage.getItem(viewModeStorageKey());
  if (mode) {
    return mode;
  } else {
    return "grid";
  }
});
const showGrid = computed(() => (groupRef.value ? groupRef.value.isA : true));
watch(showGrid, () => {
  // FIXME(nick,victor): this needs to work!
  // unfocus();
  // unhover();
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

const viewListQuery = useQuery<View[]>({
  queryKey: key(EntityKind.ViewList),
  queryFn: async () => {
    const views = await bifrostList<View[]>(args(EntityKind.ViewList));
    if (!views) return [];
    else return views;
  },
});
const viewListOptions = computed(() => {
  const list = viewListQuery.data.value ?? [];
  return list.map((l) => {
    return { value: l.id, label: l.name };
  });
});

const defaultView = computed(() =>
  viewListQuery.data.value?.find((v) => v.isDefault),
);
const selectedViewOrDefaultId = computed(() => {
  if (selectedView.value) return selectedView.value;
  if (!viewListQuery.data.value) return "";
  const view = viewListQuery.data.value.find((v) => v.isDefault);
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

const componentListRaw = useQuery<ComponentInList[]>({
  queryKey: componentQueryKey,
  queryFn: async () => {
    const arg = selectedView.value
      ? args<Listable>(EntityKind.ViewComponentList, selectedView.value)
      : args<Listable>(EntityKind.ComponentList);
    const list = await bifrostList<ComponentInList[]>(arg);
    return list ?? [];
  },
});

const componentList = computed(() => componentListRaw.data.value ?? []);
const componentsById = computed(() =>
  Object.fromEntries(componentList.value.map((c) => [c.id, c])),
);

const scrollRef = ref<HTMLDivElement>();

const filteredComponents = reactive<ComponentInList[]>([]);
const groupedComponents = computed(() => {
  const groups: Record<string, ComponentInList[]> = {};

  if (groupBySelection.value === "Qualification Status") {
    for (const component of filteredComponents) {
      const title = getQualificationSummary(component);
      groups[title] ??= [];
      groups[title]?.push(component);
    }
  }

  return groups;
});

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

    mapRef.value?.deselect();
    // FIXME(nick,victor): this needs to work!
    // unfocus();
    // unhover();
  },
  { immediate: true, deep: true },
);

const collapsingStyles = computed(() =>
  collapsingGridStyles([
    actionsRef.value?.openState,
    historyRef.value?.openState,
  ]),
);

// FIXME(nick,victor): this needs to work!
// const hoveredComponentId = computed(
//   () => filteredComponents[selectorGridPosition.value]?.id,
// );
// const interactionTargetComponentId = computed(
//   () => focusedComponentId.value ?? hoveredComponentId.value,
// );
// const constrainPosition = () => {
//   selectorGridPosition.value = Math.min(
//     filteredComponents.length - 1,
//     Math.max(-1, selectorGridPosition.value),
//   );
//   scrollCurrentTileIntoView();
// };

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
  keyEmitter.on("Delete", onBackspace);
  keyEmitter.on("/", openShortcutModal);
  keyEmitter.on("?", openShortcutModal);
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
  keyEmitter.off("Delete", onBackspace);
  keyEmitter.off("/", openShortcutModal);
  keyEmitter.off("?", openShortcutModal);
  windowResizeEmitter.off("resize", onResize);
};
const nextComponent = (wrap = false) => {
  // FIXME(nick,victor): this needs to work!
  //  if (!showGrid.value) return;
  //  if (focusedComponentId.value) unfocus();
  //  selectorGridPosition.value += 1;
  //  if (wrap && selectorGridPosition.value > filteredComponents.length - 1) {
  //    selectorGridPosition.value = -1;
  //    searchRef.value?.focus();
  //  }
  //  constrainPosition();
  //  if (CONTROL_SCHEME === "v2" && hoveredComponentId.value) {
  //    focus(hoveredComponentId.value);
  //  }
};
const previousComponent = (wrap = false) => {
  // FIXME(nick,victor): this needs to work!
  //  if (!showGrid.value) return;
  //  if (focusedComponentId.value) unfocus();
  //  selectorGridPosition.value -= 1;
  //  if (wrap) {
  //    if (selectorGridPosition.value < -1) {
  //      selectorGridPosition.value = filteredComponents.length - 1;
  //    } else if (selectorGridPosition.value < 0) {
  //      selectorGridPosition.value = -1;
  //      searchRef.value?.focus();
  //    }
  //  }
  //  constrainPosition();
  //  if (CONTROL_SCHEME === "v2" && hoveredComponentId.value) {
  //    focus(hoveredComponentId.value);
  //  }
};

const onK = (e: KeyDetails["k"]) => {
  e.preventDefault();

  // Deselect the current selection based on which screen you are on
  if (showGrid.value) {
    // FIXME(nick,victor): this needs to work!
    //    unfocus();
    //    unhover();
  } else {
    mapRef.value?.deselect();
  }

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
    // FIXME(nick,victor): this needs to work!
    //     if (selectorGridPosition.value !== -1) {
    //       if (!interactionTargetComponentId.value) return;
    //       componentContextMenuRef.value?.componentsStartErase([
    //         interactionTargetComponentId.value,
    //       ]);
    //     }
  } else {
    mapRef.value?.onE(e);
  }
};
const onD = (e: KeyDetails["d"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (e.metaKey || e.ctrlKey) {
      // FIXME(nick,victor): this needs to work!
      //      if (!interactionTargetComponentId.value) return;
      //      componentContextMenuRef.value?.componentDuplicate([
      //        interactionTargetComponentId.value,
      //      ]);
    }
  } else {
    mapRef.value?.onD(e);
  }
};

const upgrade = useUpgrade();
const onU = (e: KeyDetails["u"]) => {
  e.preventDefault();

  if (showGrid.value) {
    // FIXME(nick,victor): this needs to work!
    //    if (!interactionTargetComponentId.value) return;
    //    const targetComponent = filteredComponents.find(
    //      (comp) => comp.id === interactionTargetComponentId.value,
    //    );
    //    if (
    //      targetComponent &&
    //      upgrade(targetComponent.schemaId, targetComponent.schemaVariantId).value
    //    ) {
    //      componentContextMenuRef.value?.componentUpgrade([
    //        interactionTargetComponentId.value,
    //      ]);
    //    }
  } else {
    mapRef.value?.onU(e);
  }
};
const onBackspace = (e: KeyDetails["Backspace"] | KeyDetails["Delete"]) => {
  e.preventDefault();

  if (showGrid.value) {
    // FIXME(nick,victor): this needs to work!
    //    if (!interactionTargetComponentId.value) return;
    //    const component = componentsById.value[interactionTargetComponentId.value];
    // if (!component) return;
    // componentContextMenuRef.value?.componentsStartDelete([component]);
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
    // FIXME(nick,victor): this needs to work!
    //    if (!interactionTargetComponentId.value) return;
    //    const targetComponent = filteredComponents.find(
    //      (comp) => comp.id === interactionTargetComponentId.value,
    //    );
    //    if (targetComponent && targetComponent.toDelete) {
    //      componentContextMenuRef.value?.componentsRestore([
    //        interactionTargetComponentId.value,
    //      ]);
    //    }
  } else {
    mapRef.value?.onR(e);
  }
};
// TODO bring back arrow controls before merging
const onArrowUp = (e: KeyDetails["ArrowUp"]) => {
  //   if (CONTROL_SCHEME === "v2" && showGrid.value) return;
  //   e.preventDefault();
  //   if (showGrid.value) {
  //     if (focusedComponentId.value) unfocus();
  //     selectorGridPosition.value -= lanes.value;
  //     constrainPosition();
  //   } else {
  //     mapRef.value?.onArrowUp();
  //   }
};
const onArrowDown = (e: KeyDetails["ArrowDown"]) => {
  //   if (CONTROL_SCHEME === "v2" && showGrid.value) return;
  //   e.preventDefault();
  //   if (showGrid.value) {
  //     if (focusedComponentId.value) unfocus();
  //     if (selectorGridPosition.value === -1) {
  //       selectorGridPosition.value = 0;
  //     } else {
  //       selectorGridPosition.value += lanes.value;
  //     }
  //     constrainPosition();
  //   } else {
  //     mapRef.value?.onArrowDown();
  //   }
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
    // FIXME(nick,victor): this needs to work!
    // unfocus();
    // unhover();
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
    // FIXME(nick,victor): this needs to work!
    //  } else if (selectorGridPosition.value === -1 && !searchRef.value.isFocus) {
    //    searchRef.value.focus();
  } else {
    pageFunc(true);
  }
};
const onEnter = (e: KeyDetails["Enter"]) => {
  // FIXME(nick,victor): this needs to work!
  //  if (CONTROL_SCHEME === "v2" && focusedComponentId.value) {
  //    // enter controls the context menu, not the grid tile
  //    return;
  //  }
  e.preventDefault();
  if (!showGrid.value) {
    if (mapRef.value) {
      mapRef.value.navigateToSelectedComponent();
    }
    return;
  }

  // FIXME(nick,victor): this needs to work!
  //  if (selectorGridPosition.value !== -1) {
  //    const componentId = filteredComponents[selectorGridPosition.value]?.id;
  //    if (!componentId) return;
  //    componentInteract(componentId);
  //  }
};
const onScroll = () => {
  // FIXME(nick,victor): this needs to work!
  //  if (focusedComponentId.value && CONTROL_SCHEME === "v1") {
  //    unfocus();
  //  } else {
  //    // for the v2 interface, close the menu while scrolling
  //    componentContextMenuRef.value?.close();
  //  }
};
const fixContextMenuAfterScroll = () => {
  // For the v2 control scheme, we need to fix the context menu after scrolling
  // If the element is scrolled into view, show the menu
  // If the element is scrolled offscreen, unfocus/unhover as per v1
  if (CONTROL_SCHEME === "v1") return;
  // FIXME(nick,victor): this needs to work!
  //  else if (focusedComponentId.value) {
  //    const tileIndex = getGridTileIndexByComponentId(focusedComponentId.value);
  //    const tile = getGridTileByIndex(tileIndex);
  //    const el = tile?.$el;
  //    if (el && elementIsScrolledIntoView(el)) {
  //      focus(focusedComponentId.value);
  //    } else {
  //      unfocus();
  //      unhover();
  //    }
  //  }
};
const onResize = () => {
  // FIXME(nick,victor): this needs to work!
  //  unfocus();
  //  unhover();
};
const onClick = (e: MouseEvent) => {
  if (showGrid.value) {
    const inside =
      componentContextMenuRef.value?.contextMenuRef?.elementIsInsideMenu;
    if (inside && e.target instanceof Node && inside(e.target)) {
      return;
    }

    // general click handler for the whole page
    // any click which doesn't do this behavior should have .stop on it!
    // FIXME(nick,victor): this needs to work!
    //    unfocus();
    //    unhover();
  }
};

onMounted(() => {
  mountEmitters();
  document.addEventListener("click", onClick);
});
onBeforeUnmount(() => {
  removeEmitters();
  document.removeEventListener("click", onClick);
});

// FIXME(nick,victor): this needs to work!
// const componentInteract = (componentId: ComponentId) => {
//   if (focusedComponentId.value && CONTROL_SCHEME === "v1") {
//     componentNavigate(componentId);
//   } else {
//     focus(componentId);
//   }
// };

const openFocusedComponent = () => {
  // FIXME(nick,victor): this needs to work!
  //  if (focusedComponentId.value) {
  //    componentNavigate(focusedComponentId.value);
  //  }
};

// FIXME(nick,victor): this needs to work!
// const componentNavigate = (componentId: ComponentId) => {
//   const params = { ...route.params };
//   params.componentId = componentId;
//   router.push({
//     name: "new-hotness-component",
//     params,
//   });
// };

const addComponentModalRef = ref<InstanceType<typeof AddComponentModal>>();

const openAddComponentModal = () => {
  addComponentModalRef.value?.open();
};

const shortcutModalRef = ref<InstanceType<typeof ShortcutModal>>();

const openShortcutModal = () => {
  shortcutModalRef.value?.open();
};

const addViewModalRef = ref<InstanceType<typeof AddViewModal>>();

const openAddViewModal = () => {
  addViewModalRef.value?.open();
};

const editViewModalRef = ref<InstanceType<typeof EditViewModal>>();

const openEditViewModal = (viewId: string) => {
  // FIXME(nick): right now, we need to switch the current view to edit it because we need the query data.
  // Why? The current view may differ than the one wishing to be edited. I had a version of this where the
  // modal had an inner query to account for this scenario (essentially two different "view component list"
  // calls), but I scrapped it since the query would never load properly. Ideally, you should be able to
  // delete a view or edit it without switching to it. For now, we will survive.
  selectedView.value = viewId;

  // Handle if we are dealing with the default view first.
  if (viewId === defaultView.value?.id) {
    editViewModalRef.value?.open(viewId, false, true);
  } else {
    const canDeleteView = componentListRaw.data.value
      ? componentListRaw.data.value.length < 1
      : false;
    editViewModalRef.value?.open(viewId, canDeleteView, false);
  }
};

const componentContextMenuRef =
  ref<InstanceType<typeof ComponentContextMenu>>();

// FIXME(nick,victor): this needs to work!
// const scrollCurrentTileIntoView = () => {
//   // don't scroll if the index is out of bounds
//   if (
//     selectorGridPosition.value < 0 ||
//     selectorGridPosition.value > filteredComponents.length - 1
//   )
//     return;
//   // otherwise use the virtualizer to scroll
//   // so that even if the DOM element doesn't exist
//   // it will still work!
//   virtualList.value.scrollToIndex(
//     getRowIndexByGridTileIndex(selectorGridPosition.value),
//     { behavior: "smooth" },
//   );
// };

const onMapDeselect = () => {
  searchRef.value?.blur();
};

const storeViewMode = () => {
  if (!groupRef.value) return;

  const key = viewModeStorageKey();

  if (groupRef.value.isB) {
    localStorage.setItem(key, "grid");
  } else {
    localStorage.setItem(key, "map");
  }
};

// Group By
const groupBySelection = ref();
const groupByOptions = [
  { value: "Qualification Status", label: "Qualification Status" },
  { value: "Change Status", label: "Change Status" },
];
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
