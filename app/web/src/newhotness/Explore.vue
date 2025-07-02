<template>
  <DelayedSkeleton v-if="componentListRaw.isLoading.value" />
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
          :modelValue="selectedViewId"
          minWidthToAnchor
          placeholder="All Views"
          checkable
          :enableSecondaryAction="() => true"
          @secondaryAction="openEditViewModal"
          @update:modelValue="(val) => (selectedViewId = val)"
        >
          <template #beforeOptions>
            <DropdownMenuItem
              label="All Views"
              value="''"
              checkable
              sizeClass="h-lg px-xs pr-xs"
              :checked="selectedViewId === ''"
              @select="() => (selectedViewId = '')"
            />
          </template>
          <template #afterOptions>
            <DropdownMenuItem
              class="border-t"
              label="Add a View"
              icon="plus"
              sizeClass="h-lg px-xs pr-xs"
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
          :pills="showGrid ? ['Tab'] : undefined"
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
              :placeholder="placeholderSearchText"
              @focus="
                () => {
                  slotProps.focus();
                  mapRef?.deselect();
                  unfocus();
                }
              "
              @blur="slotProps.blur"
              @keydown.tab="(e: KeyboardEvent) => onTab(e, true)"
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
        <div v-if="showGrid" class="flex flex-row gap-xs">
          <DropdownMenuButton
            class="rounded"
            :options="groupByDropDownOptions"
            :modelValue="groupBySelection"
            placeholder="Group by"
            minWidthToAnchor
            checkable
            alwaysShowPlaceholder
            highlightWhenModelValue
            :disabled="pinnedComponentId !== undefined"
            @update:modelValue="
              (val) => (groupBySelection = groupByFromString(val))
            "
          >
            <template #beforeOptions>
              <DropdownMenuItem
                label="None"
                value="''"
                checkable
                :checked="groupBySelection === ''"
                @select="() => (groupBySelection = GroupByCriteria.None)"
              />
            </template>
          </DropdownMenuButton>
          <!--
          Something subtle here... we dynamically change highlighting for model value because we
          want the default state to be "latest to oldest" and we want it to be obvious to the user.
          Therefore, we don't highlight when it is "latest to oldest" _and_ we don't use an empty
          string for the default. Why not the latter? We want the button to also show "latest to
          oldest" next to the placeholder.
          -->
          <DropdownMenuButton
            class="rounded"
            :options="sortByDropDownOptions"
            :modelValue="sortBySelection"
            placeholder="Sort by"
            minWidthToAnchor
            checkable
            alwaysShowPlaceholder
            :highlightWhenModelValue="
              sortBySelection !== SortByCriteria.LatestToOldest
            "
            @update:modelValue="
              (val) => (sortBySelection = sortByFromString(val))
            "
          />
        </div>
      </div>
      <template v-if="showGrid">
        <div
          v-if="componentList.length === 0 && componentListRaw.isSuccess.value"
          class="flex-1 overflow-hidden flex flex-row items-center justify-center"
        >
          <EmptyState icon="component" text="No components in view" />
        </div>
        <ExploreGrid
          v-else
          ref="exploreGridRef"
          :components="sortedAndGroupedComponents"
          :focusedComponentIdx="focusedComponentIdx"
          @childClicked="componentClicked"
          @unpin="() => (pinnedComponentId = undefined)"
          @scrollend="fixContextMenuAfterScroll"
          @scroll="onScroll"
        />
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
          <EmptyState
            v-if="actionViewList.length === 0"
            icon="tools"
            text="No actions to display"
          />
          <ul v-else class="actions list">
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
            'flex-none h-12 border-t flex flex-row items-center gap-xs p-xs',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <TextPill
          v-tooltip="componentCountTooltip"
          class="flex-none rounded p-xs"
          variant="key2"
        >
          Total: {{ componentListRaw.data.value?.length ?? 0 }}
        </TextPill>
        <TextPill
          v-if="resourceCount > 0"
          v-tooltip="resourceCountTooltip"
          :class="
            clsx(
              'flex-none flex flex-row items-center gap-2xs rounded p-xs',
              themeClasses(
                'border-success-400 bg-success-100 text-black',
                'border-success-800 bg-success-900 text-white',
              ),
            )
          "
        >
          <Icon
            :class="themeClasses('text-success-600', 'text-success-400')"
            name="check-hex-outline"
            size="xs"
          />
          {{ resourceCount }}
        </TextPill>
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
      @deleted="() => (selectedViewId = '')"
    />
    <ComponentContextMenu
      ref="componentContextMenuRef"
      onGrid
      enableKeyboardControls
      @edit="navigateToFocusedComponent"
      @pin="(c) => (pinnedComponentId = c)"
    />
  </section>
</template>

<script lang="ts" setup>
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
  TextPill,
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
import { elementIsScrolledIntoView } from "@/newhotness/logic_composables/dom_funcs";
import { ActionState } from "@/api/sdf/dal/action";
import Map from "./Map.vue";
import { collapsingGridStyles } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import { getQualificationStatus } from "./ComponentQualificationStatus.vue";
import ActionCard from "./ActionCard.vue";
import FuncRunList from "./FuncRunList.vue";
import { assertIsDefined, Context, ExploreContext } from "./types";
import {
  KeyDetails,
  keyEmitter,
  MouseDetails,
  mouseEmitter,
  windowResizeEmitter,
} from "./logic_composables/emitters";
import TabGroupToggle from "./layout_components/TabGroupToggle.vue";
import { SelectionsInQueryString } from "./Workspace.vue";
import AddComponentModal from "./AddComponentModal.vue";
import AddViewModal from "./AddViewModal.vue";
import EditViewModal from "./EditViewModal.vue";
import ComponentContextMenu from "./ComponentContextMenu.vue";
import EmptyState from "./EmptyState.vue";
import ShortcutModal from "./ShortcutModal.vue";
import { useUpgrade } from "./logic_composables/upgrade";
import ExploreGrid from "./explore_grid/ExploreGrid.vue";
import { useConnections } from "./logic_composables/connections";
import DelayedSkeleton from "./skeletons/DelayedSkeleton.vue";

const router = useRouter();
const route = useRoute();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const key = useMakeKey();
const args = useMakeArgs();

const VIEW_MODE_LOCAL_STORAGE_KEY = "newhotness-view-mode";
const viewModeStorageKey = () =>
  `${VIEW_MODE_LOCAL_STORAGE_KEY}: ${ctx.changeSetId}`;

const groupRef = ref<InstanceType<typeof TabGroupToggle>>();
const actionsRef = ref<typeof CollapsingGridItem>();
const historyRef = ref<typeof CollapsingGridItem>();
const mapRef = ref<InstanceType<typeof Map>>();
const exploreGridRef = ref<InstanceType<typeof ExploreGrid>>();
const componentContextMenuRef =
  ref<InstanceType<typeof ComponentContextMenu>>();

const collapsingStyles = computed(() =>
  collapsingGridStyles([
    actionsRef.value?.openState,
    historyRef.value?.openState,
  ]),
);

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
  unfocus();
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.grid;
  if (showGrid.value) query.grid = "1";
  else query.map = "1";
  router.push({ query });
});

// ================================================================================================
// SETUP THE FILTERED COMPONENTS REACTIVE AND UPGRADEABLES
const filteredComponents = reactive<ComponentInList[]>([]);
const upgrade = useUpgrade();
const upgradeableComponents = computed(() => {
  const set: Set<ComponentId> = new Set();

  // TODO(nick): try to swap this with the component list to see if we recompute this less
  // frequently. This is not a problem today, but could be tomorrow.
  for (const component of filteredComponents) {
    // This needs to be split out into a variable for reactivity. Keep this here or drown in
    // sorrow and suffering. Relevant pull request: https://github.com/systeminit/si/pull/6483
    const canUpgrade = upgrade(
      component.schemaId,
      component.schemaVariantId,
    ).value;
    if (canUpgrade) {
      set.add(component.id);
    }
  }
  return set;
});

// ================================================================================================
// VIEWS
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

  // This is ID-sorted in the backend, not name-sorted.
  return list
    .map((l) => ({ value: l.id, label: l.name }))
    .sort((a, b) => a.label.localeCompare(b.label));
});

const defaultView = computed(() =>
  viewListQuery.data.value?.find((v) => v.isDefault),
);
const selectedViewId = ref<string | undefined>(undefined);
const selectedViewOrDefaultId = computed(() => {
  if (selectedViewId.value) return selectedViewId.value;
  if (!viewListQuery.data.value) return "";
  const view = viewListQuery.data.value.find((v) => v.isDefault);
  if (!view) return "";
  return view.id;
});

// Store the viewId in the URL if it's not the default view or all views
watch([selectedViewId], () => {
  unfocus();
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.viewId;
  query.grid = "1";
  if (
    selectedViewId.value !== "" &&
    selectedViewId.value !== defaultView.value?.id
  ) {
    query.viewId = selectedViewOrDefaultId.value;
  }
  router.push({
    query,
  });
});

// We need to check if the change set has been changed
// and if it has and the selected view doesn't exist in that
// new change set, then we default back to 'All Views'
watch(
  ctx?.changeSetId,
  () => {
    if (
      !viewListQuery.data.value ||
      !viewListQuery.data.value.find((v) => v.id === selectedViewId.value)
    ) {
      selectedViewId.value = "";
    }
  },
  { immediate: true },
);

// ================================================================================================
// COMPONENT PINNING
//
// You might wonder why the entire component isn't the ref. It was originally. The component
// context menu emitted the entire object. The problem is that it's possible to have a pinned
// component in the query string that no longer exists or has been filtered out. Therefore, we need
// to compute the component from the ID rather than the other way around.
const pinnedComponentId = ref<ComponentId | undefined>(undefined);

watch([pinnedComponentId], () => {
  // First, make sure we unfocus.
  unfocus();

  // Update the query of the route (allowing for URL links) when the pinned component changes.
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.pinned;

  query.grid = "1";

  if (pinnedComponentId.value !== undefined) {
    query.pinned = pinnedComponentId.value;
  }

  router.push({
    query,
  });
});

// ================================================================================================
// EXPLORE CONTEXT
const exploreContext = computed<ExploreContext>(() => {
  return {
    viewId: selectedViewOrDefaultId,
    upgradeableComponents,
  };
});

provide("EXPLORE_CONTEXT", exploreContext.value);

// ================================================================================================
// ACTIONS INFORMATION FOR GROUP BY
const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

interface ComponentsHaveActionsWithState {
  failed: Set<ComponentId>;
  running: Set<ComponentId>;
}

const componentsHaveActionsWithState = computed(() => {
  const results: ComponentsHaveActionsWithState = {
    failed: new Set(),
    running: new Set(),
  };
  for (const action of actionViewList.value) {
    if (!action.componentId) continue;
    if (action.state === ActionState.Failed) {
      results.failed.add(action.componentId);
    } else if (action.state === ActionState.Running) {
      results.running.add(action.componentId);
    }
  }
  return results;
});

// ================================================================================================
// ALL COMPONENTS AVAILABLE FOR USE, INCLUDING VIEWS AND PINNING
const componentListQueryKind = computed(() =>
  selectedViewId.value
    ? EntityKind.ViewComponentList
    : EntityKind.ComponentList,
);
const componentListQueryId = computed(() =>
  selectedViewId.value ? selectedViewId.value : ctx.workspacePk.value,
);
const componentQueryKey = key(componentListQueryKind, componentListQueryId);
const componentListRaw = useQuery<ComponentInList[]>({
  queryKey: componentQueryKey,
  queryFn: async () => {
    const arg = selectedViewId.value
      ? args<Listable>(EntityKind.ViewComponentList, selectedViewId.value)
      : args<Listable>(EntityKind.ComponentList);
    const list = await bifrostList<ComponentInList[]>(arg);
    return list ?? [];
  },
});
const placeholderSearchText = computed(
  () => `Search across ${componentListRaw.data.value?.length ?? 0} Components`,
);
const componentListUnchecked = computed(
  () => componentListRaw.data.value ?? [],
);
const pinnedComponent = computed(() =>
  componentListUnchecked.value.find((c) => c.id === pinnedComponentId.value),
);
const connectionsGetter = useConnections();
const pinnedComponentConnections = computed(() =>
  // This is critical. We only want to get the connections if we found the pinned component. The ID
  // could have been provided via URL and the component may not exist anymore. In short, it is
  // totally okay to have "pinnedComponentId" be populated, but "pinnedComponent" not be.
  pinnedComponentId.value && pinnedComponent.value
    ? connectionsGetter(pinnedComponentId.value).value
    : undefined,
);
const pinnedComponentConnectionSets = computed(() => {
  const incoming = new Set(
    pinnedComponentConnections.value?.incoming.map((c) => c.componentId) ?? [],
  );
  const outgoing = new Set(
    pinnedComponentConnections.value?.outgoing.map((c) => c.componentId) ?? [],
  );
  return {
    incoming,
    outgoing,
  };
});
const componentList = computed(() => {
  // If we aren't dealing with pinning, return the standard list.
  if (!pinnedComponent.value || !pinnedComponentConnections.value)
    return componentListUnchecked.value;

  // If we are dealing with pinning, we are only concerned with relevant components.
  const relevantComponentsWhenPinning = [pinnedComponent.value];
  for (const component of componentListUnchecked.value) {
    if (
      pinnedComponentConnectionSets.value.incoming.has(component.id) ||
      pinnedComponentConnectionSets.value.outgoing.has(component.id)
    ) {
      relevantComponentsWhenPinning.push(component);
    }
  }
  return relevantComponentsWhenPinning;
});

const resourceCount = computed(
  () => componentListUnchecked.value.filter((c) => c.hasResource).length ?? 0,
);
const resourceCountTooltip = "Components with resources";
const componentCountTooltip = "Total components in selected view";

// ================================================================================================
// HANDLE FILTERING, SORTING, GROUPING, ETC. FOR THE COMPUTED COMPONENT LIST
//
// Order of operations...
//   1) setup the reactive filtered components array as it needs to be initialized upfront
//   2) compute the final component list (accounting for pinning and views)
//   3) react to the search bar to populate filtered components (upgradeable reacts to this at the
//      time of writing)
//   4) sort and group the filtered components, which can be used for the grid and indexing
const sortedAndGroupedComponents = computed(() => {
  let groups: Record<string, ComponentInList[]> = {};

  // First, always sort by latest to oldest. This relies on the fact ULIDs are time-based.
  let components = filteredComponents;
  components.sort((a, b) => b.id.localeCompare(a.id));

  // Second, perform any secondary sorts, if applicable. This relies on the fact that the
  // components are already sorted.
  if (sortBySelection.value === SortByCriteria.FailingActions) {
    const failed = [];
    const theRest = [];
    for (const component of components) {
      if (componentsHaveActionsWithState.value.failed.has(component.id)) {
        failed.push(component);
      } else {
        theRest.push(component);
      }
    }
    components = [...failed, ...theRest];
  } else if (sortBySelection.value === SortByCriteria.RunningActions) {
    const running = [];
    const theRest = [];
    for (const component of components) {
      if (componentsHaveActionsWithState.value.running.has(component.id)) {
        running.push(component);
      } else {
        theRest.push(component);
      }
    }
    components = [...running, ...theRest];
  }

  // Third, separate the components into groups. There will always be at least one group.
  if (pinnedComponent.value) {
    groups = {
      Pinned: [pinnedComponent.value],
      "Incoming connections": [],
      "Outgoing connections": [],
    };
    for (const component of components) {
      // This is subtle, but we do not use "else-if". This should have no opinion on whether or not
      // something can be both an input or an output.
      if (pinnedComponentConnectionSets.value.incoming.has(component.id)) {
        groups["Incoming connections"] ??= [];
        groups["Incoming connections"].push(component);
      }
      if (pinnedComponentConnectionSets.value.outgoing.has(component.id)) {
        groups["Outgoing connections"] ??= [];
        groups["Outgoing connections"].push(component);
      }
    }
  } else if (groupBySelection.value === "Diff Status") {
    groups = {
      "With Diffs": [],
      "No Diffs": [],
    };
    for (const component of components) {
      const title = component.diffCount === 0 ? "No Diffs" : "With Diffs";
      groups[title]?.push(component);
    }
  } else if (groupBySelection.value === "Qualification Status") {
    groups = {
      "Failed qualifications": [],
      Warnings: [],
      "Passed qualifications": [],
      "Unknown qualification status": [],
    };
    for (const component of components) {
      const title = getQualificationStatusTitle(component);
      groups[title] ??= [];
      groups[title]?.push(component);
    }
  } else if (groupBySelection.value === "Upgradeable") {
    groups = {
      Upgradeable: [],
      "Up to date": [],
    };
    for (const component of components) {
      const title = upgradeableComponents.value.has(component.id)
        ? "Upgradeable"
        : "Up to date";
      groups[title]?.push(component);
    }
  } else {
    groups[""] = components;
  }

  return groups;
});

// ================================================================================================
// THE SEARCH BAR AND FILTERING
const searchString = ref<string>("");
const debouncedSearchString = ref<string>("");
const computedSearchString = computed(() => debouncedSearchString.value);

// send this down to any components that might use it
provide("SEARCH", computedSearchString);

watch(
  () => [debouncedSearchString.value, componentList.value],
  () => {
    if (!debouncedSearchString.value) {
      filteredComponents.splice(0, Infinity, ...componentList.value);
      return;
    }

    const fzf = new Fzf(componentList.value, {
      casing: "case-insensitive",
      selector: (c) =>
        `${c.name} ${c.schemaVariantName} ${c.schemaName} ${c.schemaCategory} ${c.schemaId} ${c.id}`,
    });

    const results = fzf.find(debouncedSearchString.value);
    filteredComponents.splice(0, Infinity, ...results.map((fz) => fz.item));

    mapRef.value?.deselect();
    unfocus();
  },
  { immediate: true, deep: true },
);

// Debounce the search string updates to avoid expensive filtering on every keystroke
const updateDebouncedSearch = _.debounce(
  (value: string) => {
    debouncedSearchString.value = value;
  },
  500,
  { trailing: true, leading: false },
);

// Watch for changes to fuzzySearchString and update the debounced version
watch(searchString, (newValue, oldValue) => {
  if (oldValue === "" && newValue === null) {
    // this is not a real change in the search string!
    return;
  }
  updateDebouncedSearch(newValue);
  mapRef.value?.deselect();
  unfocus();
});

// ================================================================================================
// FOCUSING, TABBING, ETC.
const focusedComponentIdx = ref<number>(-1);
const focusedComponent = computed(() => exploreGridRef.value?.focusedComponent);
const focusedComponentIsPinned = computed(() => {
  if (!focusedComponent.value) return false;
  return focusedComponent.value.id === pinnedComponentId.value;
});

const nextComponent = (wrap = false) => {
  if (!showGrid.value) return;

  if (focusedComponentIdx.value === undefined) {
    focusedComponentIdx.value = -1;
    return;
  }

  focusedComponentIdx.value += 1;

  if (focusedComponentIdx.value > filteredComponents.length - 1) {
    if (wrap) {
      focusedComponentIdx.value = -1;
    } else {
      focusedComponentIdx.value = filteredComponents.length - 1;
    }
  }
};
const previousComponent = (wrap = false) => {
  if (!showGrid.value) return;

  if (focusedComponentIdx.value === undefined) {
    focusedComponentIdx.value = -1;
    return;
  }

  let desiredIdx = focusedComponentIdx.value - 1;

  if (desiredIdx < -1) {
    if (wrap) {
      desiredIdx = filteredComponents.length - 1;
    } else {
      desiredIdx = -1;
    }
  }

  focusedComponentIdx.value = desiredIdx;
};

watch([focusedComponentIdx], () => {
  if (focusedComponentIdx.value === -1) {
    searchRef.value?.focus();
  }
});

const focusedGridComponentRef = computed(() =>
  focusedComponentIdx.value > -1
    ? exploreGridRef.value?.getGridComponentRefByIndex(
        focusedComponentIdx.value,
      )
    : undefined,
);

const onFocus = () => {
  // If we focus on the pinned component, do not bring up the context menu.
  if (
    focusedGridComponentRef.value &&
    focusedComponent.value &&
    !focusedComponentIsPinned.value
  ) {
    componentContextMenuRef.value?.open(focusedGridComponentRef.value, [
      focusedComponent.value,
    ]);
  }
};
const unfocus = () => {
  focusedComponentIdx.value = -1;
  componentContextMenuRef.value?.close();
};

// ================================================================================================
// CLICKING AND NAVIGATION
const componentClicked = (
  e: MouseEvent,
  componentId: ComponentId,
  componentIdx: number,
) => {
  e.preventDefault();
  if (e.button === 0) {
    componentNavigate(componentId);
  } else if (e.button === 2) {
    focusedComponentIdx.value = componentIdx;
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

watch([focusedComponent], () => {
  if (!focusedComponent.value) return;

  onFocus();
});

// ================================================================================================
// KEYBOARD NAVIGATION
const searchRef = ref<InstanceType<typeof VormInput>>();
const mountEmitters = () => {
  removeEmitters();
  keyEmitter.on("c", onC);
  keyEmitter.on("k", onK);
  keyEmitter.on("n", onN);
  keyEmitter.on("e", onE);
  keyEmitter.on("d", onD);
  keyEmitter.on("u", onU);
  keyEmitter.on("r", onR);
  keyEmitter.on("p", onP);
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
  keyEmitter.off("c", onC);
  keyEmitter.off("k", onK);
  keyEmitter.off("n", onN);
  keyEmitter.off("e", onE);
  keyEmitter.off("d", onD);
  keyEmitter.off("u", onU);
  keyEmitter.off("r", onR);
  keyEmitter.off("p", onP);
  keyEmitter.off("Enter", onEnter);
  keyEmitter.off("Tab", onTab);
  keyEmitter.off("Escape", onEscape);
  keyEmitter.off("Backspace", onBackspace);
  keyEmitter.off("Delete", onBackspace);
  keyEmitter.off("/", openShortcutModal);
  keyEmitter.off("?", openShortcutModal);
  windowResizeEmitter.off("resize", onResize);
};

const onC = (e: KeyDetails["c"]) => {
  e.preventDefault();
  if (e.metaKey || e.ctrlKey) return;
  emit("openChangesetModal");
};

const onK = (e: KeyDetails["k"]) => {
  e.preventDefault();

  // Deselect the current selection based on which screen you are on
  if (showGrid.value) {
    unfocus();
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
    if (!focusedComponent.value) return;

    componentContextMenuRef.value?.componentsStartErase([
      focusedComponent.value?.id,
    ]);
  } else {
    mapRef.value?.onE(e);
  }
};
const onD = (e: KeyDetails["d"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (e.metaKey || e.ctrlKey) {
      if (!focusedComponent.value) return;
      componentContextMenuRef.value?.componentDuplicate([
        focusedComponent.value.id,
      ]);
    }
  } else {
    mapRef.value?.onD(e);
  }
};
const onP = (e: KeyDetails["p"]) => {
  e.preventDefault();
  if (showGrid.value) {
    if (!focusedComponent.value) return;

    // We do not need the context menu to pin and unpin.
    if (focusedComponentIsPinned.value) {
      pinnedComponentId.value = undefined;
    } else {
      pinnedComponentId.value = focusedComponent.value.id;
    }
  } else {
    mapRef.value?.onP(e);
  }
};
const onU = (e: KeyDetails["u"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!focusedComponent.value) return;

    if (upgradeableComponents.value.has(focusedComponent.value.id)) {
      componentContextMenuRef.value?.componentUpgrade([
        focusedComponent.value.id,
      ]);
    }
  } else {
    mapRef.value?.onU(e);
  }
};
const onBackspace = (e: KeyDetails["Backspace"] | KeyDetails["Delete"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!focusedComponent.value) return;
    componentContextMenuRef.value?.componentsStartDelete([
      focusedComponent.value,
    ]);
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
    if (!focusedComponent.value) return;
    if (focusedComponent.value.toDelete) {
      componentContextMenuRef.value?.componentsRestore([
        focusedComponent.value.id,
      ]);
    }
  } else {
    mapRef.value?.onR(e);
  }
};
const onEscape = () => {
  if (showGrid.value) {
    searchRef.value?.blur();
    unfocus();
  } else {
    mapRef.value?.onEscape();
  }
};

const onTab = (e: KeyDetails["Tab"], blurSearch = false) => {
  // FIXME(victor) Don't execute this if a modal is open
  e.preventDefault();
  if (!showGrid.value) return; // no tab behavior on the map yet

  const pageFunc = e.shiftKey ? previousComponent : nextComponent;
  if (!searchRef.value) return;
  else if (blurSearch) {
    searchRef.value.blur();
    pageFunc(true);
  } else if (focusedComponentIdx.value === -1 && !searchRef.value.isFocus) {
    searchRef.value.focus();
  } else {
    pageFunc(true);
  }
};

const onEnter = (e: KeyDetails["Enter"]) => {
  // If there is a focused component, we know we may have to ignore the "ENTER" key press.
  if (focusedComponentIdx.value !== undefined) {
    // If the focused component is actually a component (and not the search bar), and it is not a
    // pinned component, then we ignore the "ENTER" key press since the component context menu will
    // pop up.
    if (focusedComponentIdx.value !== -1 && !focusedComponentIsPinned.value) {
      return;
    }
  }

  e.preventDefault();

  // If dealing with the map view, use its navigation and return immediately.
  if (!showGrid.value) {
    if (mapRef.value) {
      mapRef.value.navigateToSelectedComponent();
    }
    return;
  }

  navigateToFocusedComponent();
};

// ================================================================================================
// SCROLLING AND CLICKING
const onScroll = () => {
  componentContextMenuRef.value?.close();
};

const fixContextMenuAfterScroll = () => {
  // We need to fix the context menu after scrolling!
  // If the element is scrolled into view, show the menu
  // If the element is scrolled offscreen, unfocus and reset selected component index
  const el = focusedGridComponentRef.value?.$el;
  if (el && elementIsScrolledIntoView(el)) {
    onFocus();
  } else {
    unfocus();
  }
};
const onResize = () => {
  unfocus();
};

// general click handler for the whole page
// any click which doesn't do this behavior should have .stop on it!
const onClick = (e: MouseDetails["click"]) => {
  if (showGrid.value) {
    const inside =
      componentContextMenuRef.value?.contextMenuRef?.elementIsInsideMenu;
    if (inside && e.target instanceof Node && inside(e.target)) {
      return;
    }

    unfocus();
  }
};

// ================================================================================================
// MOUNTING AND URL QUERY HANDLING
const setSelectionsFromQuery = () => {
  const query: SelectionsInQueryString = router.currentRoute.value?.query;

  switch (query.sortBy) {
    case "failingactions":
      sortBySelection.value = SortByCriteria.FailingActions;
      break;
    case "runningactions":
      sortBySelection.value = SortByCriteria.RunningActions;
      break;
    case undefined:
    default:
      sortBySelection.value = SortByCriteria.LatestToOldest;
      break;
  }

  switch (query.groupBy) {
    case "diffstatus":
      groupBySelection.value = GroupByCriteria.Diff;
      break;
    case "qualificationstatus":
      groupBySelection.value = GroupByCriteria.Qualification;
      break;
    case "upgradeable":
      groupBySelection.value = GroupByCriteria.Upgrade;
      break;
    case undefined:
    default:
      groupBySelection.value = GroupByCriteria.None;
      break;
  }

  if (query.pinned !== undefined) {
    pinnedComponentId.value = query.pinned;
  }

  if (query.viewId !== undefined) {
    selectedViewId.value = query.viewId;
  }
};

onMounted(() => {
  mountEmitters();
  mouseEmitter.on("click", onClick);
  setSelectionsFromQuery(); // sort by, group by, pinning, etc. on mount
});
onBeforeUnmount(() => {
  removeEmitters();
  mouseEmitter.off("click", onClick);
});
watch([router.currentRoute], setSelectionsFromQuery);

// ================================================================================================
// THIS FUNCTION IS LOST AND NEEDS A HOME
const navigateToFocusedComponent = () => {
  if (focusedComponent.value) {
    componentNavigate(focusedComponent.value.id);
  }
};

// ================================================================================================
// MODAL REFS
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

const openEditViewModal = (option: { value: string; label: string }) => {
  const viewId = option.value;
  const viewName = option.label;
  const isDefaultView = viewId === defaultView.value?.id;
  editViewModalRef.value?.open(viewId, viewName, isDefaultView);
};

const onMapDeselect = () => {
  searchRef.value?.blur();
};

// ================================================================================================
// GROUP BY STUFF
const storeViewMode = () => {
  if (!groupRef.value) return;

  const key = viewModeStorageKey();

  if (groupRef.value.isB) {
    localStorage.setItem(key, "grid");
  } else {
    localStorage.setItem(key, "map");
  }
};

export type GroupByUrlQuery =
  | "diffstatus"
  | "qualificationstatus"
  | "upgradeable";

enum GroupByCriteria {
  Diff = "Diff Status",
  Upgrade = "Upgradeable",
  Qualification = "Qualification Status",
  None = "",
}

const groupByFromString = (s: string): GroupByCriteria => {
  const key = (
    _.keys(GroupByCriteria) as (keyof typeof GroupByCriteria)[]
  ).find((k) => GroupByCriteria[k] === s);

  if (!key) return GroupByCriteria.None;
  else return GroupByCriteria[key];
};

const groupBySelection = ref<GroupByCriteria>(GroupByCriteria.None);
const groupByDropDownOptions = [
  { value: GroupByCriteria.Diff, label: "Diff Status" },
  { value: GroupByCriteria.Qualification, label: "Qualification Status" },
  { value: GroupByCriteria.Upgrade, label: "Upgradeable" },
];

watch([groupBySelection], () => {
  // First, make sure we unfocus.
  unfocus();

  // Update the query of the route (allowing for URL links) when the group by selection change.
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.groupBy;

  query.grid = "1";

  if (groupBySelection.value === GroupByCriteria.Diff) {
    query.groupBy = "diffstatus";
  } else if (groupBySelection.value === GroupByCriteria.Qualification) {
    query.groupBy = "qualificationstatus";
  } else if (groupBySelection.value === GroupByCriteria.Upgrade) {
    query.groupBy = "upgradeable";
  }

  router.push({
    query,
  });
});

const getQualificationStatusTitle = (component: ComponentInList) => {
  const status = getQualificationStatus(component);
  switch (status) {
    case "success":
      return "Passed qualifications";
    case "failure":
      return "Failed qualifications";
    case "warning":
      return "Warnings";
    default:
      return "Unknown qualification status";
  }
};

// ================================================================================================
// SORT BY STUFF
export type SortByUrlQuery = "failingactions" | "runningactions";

enum SortByCriteria {
  FailingActions = "Failing actions",
  RunningActions = "Running actions",
  LatestToOldest = "Latest to oldest",
}

const sortByFromString = (s: string): SortByCriteria => {
  const key = (_.keys(SortByCriteria) as (keyof typeof SortByCriteria)[]).find(
    (k) => SortByCriteria[k] === s,
  );

  if (!key) return SortByCriteria.LatestToOldest;
  else return SortByCriteria[key];
};

const sortBySelection = ref<SortByCriteria>(SortByCriteria.LatestToOldest);
const sortByDropDownOptions = [
  { value: SortByCriteria.LatestToOldest, label: "Latest to oldest" },
  { value: SortByCriteria.FailingActions, label: "Failing actions" },
  { value: SortByCriteria.RunningActions, label: "Running actions" },
];

watch([sortBySelection], () => {
  // First, make sure we unfocus.
  unfocus();

  // Update the query of the route (allowing for URL links) when the sort by selection changes.
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.sortBy;

  query.grid = "1";

  if (sortBySelection.value === SortByCriteria.FailingActions) {
    query.sortBy = "failingactions";
  } else if (sortBySelection.value === SortByCriteria.RunningActions) {
    query.sortBy = "runningactions";
  }

  router.push({
    query,
  });
});

// ================================================================================================
// EMITS AND THE REST
const emit = defineEmits<{
  (e: "openChangesetModal"): void;
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
