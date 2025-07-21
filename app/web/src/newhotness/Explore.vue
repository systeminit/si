<template>
  <section :class="clsx('grid h-full', showGrid ? 'explore' : 'map')">
    <!-- Left column -->
    <!-- 12 pixel padding to align with the SI logo -->
    <div
      class="main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]"
    >
      <!-- Search and filters -->
      <ExploreSearchBarSkeleton v-if="showSkeleton" />
      <template v-else>
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
                :sizeClass="tw`px-xs pr-xs h-[28px]`"
                :checked="selectedViewId === ''"
                @select="() => (selectedViewId = '')"
              />
            </template>
            <template #afterOptions>
              <DropdownMenuItem
                class="border-t"
                label="Add a View"
                icon="plus"
                :sizeClass="tw`px-xs pr-xs h-[28px]`"
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
                    clearSelection();
                    mapRef?.deselect();
                    slotProps.focus();
                    focusedComponentIdx = -1;
                  }
                "
                @blur="slotProps.blur"
                @keydown.tab="(e: KeyboardEvent) => onTab(e, true)"
                @keydown.esc="onEscape"
              />
            </template>
          </InstructiveVormInput>
        </div>
        <div
          class="flex-none flex flex-row items-center gap-xs justify-between"
        >
          <TabGroupToggle
            ref="groupRef"
            :aOrB="urlGridOrMap === 'grid'"
            @toggle="storeViewMode"
          >
            <template #a="{ selected, toggle }">
              <ExploreModeTile
                icon="grid"
                label="Grid"
                :selected="selected"
                @toggle="toggle"
              />
            </template>
            <template #b="{ selected, toggle }">
              <ExploreModeTile
                icon="map"
                label="Map (Model)"
                :selected="selected"
                @toggle="toggle"
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
      </template>

      <template v-if="showGrid">
        <ExploreGridSkeleton v-if="showSkeleton" />
        <template v-else>
          <div
            v-if="
              componentList.length === 0 && componentListQuery.isSuccess.value
            "
            class="flex-1 gap-sm overflow-hidden flex flex-col items-center justify-center"
          >
            <div class="grow flex items-center justify-center">
              <EmptyState
                icon="logo-si"
                iconSize="lg"
                iconNoBg
                text="Manage your infrastructure in a clear, controlled, and intelligent way."
              >
                <template #secondary>
                  <div class="flex flex-row gap-sm">
                    <VButton
                      label="Check our how-to guide"
                      tone="action"
                      size="sm"
                      href="https://docs.systeminit.com/how-tos/"
                      target="_blank"
                    />
                    <VButton
                      label="API guidelines"
                      tone="neutral"
                      size="sm"
                      href="https://docs.systeminit.com/reference/public-api"
                      target="_blank"
                    />
                  </div>
                </template>
              </EmptyState>
            </div>
            <div
              :class="
                clsx(
                  'shrink-0 flex flex-col items-center py-md px-lg w-full max-w-6xl mb-9 gap-3  text-center',
                  themeClasses('bg-neutral-300', 'bg-neutral-800'),
                )
              "
            >
              <span class="font-bold">Explore other ways to get started</span>
              <span
                >Prefer a different starting point? Here are a few other ways to
                jump in and explore:
              </span>
              <div
                :class="
                  clsx(
                    'flex flex-row gap-md mt-sm hover:children:underline',
                    themeClasses(
                      'text-neutral-700 hover:children:text-black',
                      'text-neutral-300 hover:children:text-white',
                    ),
                  )
                "
              >
                <a
                  href="https://www.systeminit.com/?modal=demo"
                  target="_blank"
                >
                  Schedule a demo
                </a>
                <a href="https://discord.gg/system-init" target="_blank">
                  Join the community
                </a>
                <a
                  :href="`https://auth.systeminit.com/workspace/${ctx?.workspacePk.value}`"
                  target="_blank"
                >
                  Invite a team mate
                </a>
              </div>
            </div>
          </div>
          <ExploreGrid
            v-else
            ref="exploreGridRef"
            :components="sortedAndGroupedComponents"
            :focusedComponentIdx="focusedComponentIdx"
            :selectedComponentIndexes="selectedComponentIndexes"
            :componentsWithFailedActions="componentsHaveActionsWithState.failed"
            :componentsWithRunningActions="
              componentsHaveActionsWithState.running
            "
            :componentsPendingActionNames="componentsPendingActionNames"
            @childClicked="componentClicked"
            @childSelect="selectComponent"
            @childDeselect="deselectComponent"
            @childHover="
              (componentId) => {
                if (componentsHaveActionsWithState.failed.has(componentId)) {
                  hoveredComponentId = componentId;
                }
              }
            "
            @childUnhover="() => (hoveredComponentId = undefined)"
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
      </template>
      <Map
        v-else
        ref="mapRef"
        :active="!showGrid"
        :components="filteredComponents"
        :componentsWithFailedActions="componentsHaveActionsWithState.failed"
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
      <!-- Skeleton -->
      <ExploreRightColumnSkeleton v-if="showSkeleton" />
      <template v-else>
        <div
          class="grow grid grid-rows-subgrid min-h-0"
          :style="collapsingStyles"
        >
          <CollapsingGridItem ref="actionsRef">
            <template #header>Actions</template>
            <template #headerIconsRight>
              <PillCounter :count="actionViewList.length" class="text-sm" />
            </template>
            <ActionQueueList
              :actionViewList="actionViewList"
              :highlightedActionIds="highlightedActionIds"
            />
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
            Total: {{ componentList.length }}
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
      </template>
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
      @clearSelected="clearSelection"
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
import { useRoute, useRouter } from "vue-router";
import {
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
  PillCounter,
  TextPill,
  themeClasses,
  VButton,
  VormInput,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { Fzf } from "fzf";
import { tw } from "@si/vue-lib";
import {
  bifrost,
  bifrostList,
  bifrostQueryAttributes,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  ComponentInList,
  EntityKind,
  View,
} from "@/workers/types/entity_kind_types";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import { Listable } from "@/workers/types/dbinterface";
import { elementIsScrolledIntoView } from "@/newhotness/logic_composables/dom_funcs";
import { ActionState } from "@/api/sdf/dal/action";
import ExploreSearchBarSkeleton from "@/newhotness/skeletons/ExploreSearchBarSkeleton.vue";
import ExploreGridSkeleton from "@/newhotness/skeletons/ExploreGridSkeleton.vue";
import ExploreRightColumnSkeleton from "@/newhotness/skeletons/ExploreRightColumnSkeleton.vue";
import Map from "./Map.vue";
import { collapsingGridStyles, preserveExploreState } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import { getQualificationStatus } from "./ComponentQualificationStatus.vue";
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
import ExploreModeTile from "./ExploreModeTile.vue";
import ActionQueueList from "./ActionQueueList.vue";
import { parseSearch, SearchTerms } from "./logic_composables/search";

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

const urlGridOrMap = computed((): "grid" | "map" => {
  const q: SelectionsInQueryString = router.currentRoute.value?.query;
  const keys = Object.keys(q);
  if (keys.includes("grid")) return "grid";
  if (keys.includes("map")) return "map";
  const mode = localStorage.getItem(viewModeStorageKey());
  if (mode) {
    return mode as "grid" | "map";
  } else {
    return "grid";
  }
});

// Since we won't always have the group ref, let the url control showGrid
const showGrid = computed(() => urlGridOrMap.value === "grid");
const gridMapSwitcherValue = computed(
  () => groupRef.value && groupRef.value.isA,
);

watch(gridMapSwitcherValue, (newShowGrid) => {
  // If this is nil, groupRef is unmounted,and we don't care about the change.
  if (_.isNil(newShowGrid)) return;
  clearSelection();
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.grid;
  if (newShowGrid) query.grid = "1";
  else query.map = "1";
  router.push({ query });
});

// ================================================================================================
// SETUP THE FILTERED COMPONENTS REACTIVE AND UPGRADEABLES
const upgrade = useUpgrade();
const upgradeableComponentIds = computed(() => {
  const set: Set<ComponentId> = new Set();

  // TODO(nick): try to swap this with the component list to see if we recompute this less
  // frequently. This is not a problem today, but could be tomorrow.
  for (const component of filteredComponents.value) {
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
// This is the list of all components that get rendered but the explore grid, considering the filtering and grouping
// done here and any removals caused by collapsing by the ExploreGrid
const allVisibleComponents = computed(
  () => exploreGridRef.value?.allVisibleComponents ?? [],
);

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
  delete query.viewId;
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

// Track hovered component for highlighting failed actions
const hoveredComponentId = ref<ComponentId | undefined>(undefined);

watch([pinnedComponentId], () => {
  // First, make sure we clear any selection.
  clearSelection();

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
// SKELETON BEHAVIOR

const showSkeleton = computed(
  () =>
    componentListQuery.isLoading.value ||
    (!_.isNil(mapRef.value) && mapRef.value.isLoading),
);

// ================================================================================================
// EXPLORE CONTEXT
const exploreContext = computed<ExploreContext>(() => {
  return {
    viewId: selectedViewOrDefaultId,
    upgradeableComponents: upgradeableComponentIds,
    showSkeleton,
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

// Map component IDs to their pending action counts by type (can be multiple)
const componentsPendingActionNames = computed(() => {
  const results = new globalThis.Map<
    ComponentId,
    Record<string, { count: number; hasFailed: boolean }>
  >();
  for (const action of actionViewList.value) {
    if (!action.componentId) continue;
    // All action states are considered "pending" and should be shown
    if (!results.has(action.componentId)) {
      results.set(action.componentId, {});
    }
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const actionCounts = results.get(action.componentId)!;

    // Group Other actions with Manual
    let actionName = action.name;
    if (actionName.toLowerCase() === "other") {
      actionName = "Manual";
    }

    if (!actionCounts[actionName]) {
      actionCounts[actionName] = { count: 0, hasFailed: false };
    }

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    actionCounts[actionName]!.count += 1;

    // Track if any action in this group has failed
    if (action.state === ActionState.Failed) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      actionCounts[actionName]!.hasFailed = true;
    }
  }
  return results;
});

// Calculate which actions should be highlighted based on hovered component (only failed actions)
const highlightedActionIds = computed(() => {
  if (!hoveredComponentId.value) return new Set<string>();

  const highlightedIds = new Set<string>();
  for (const action of actionViewList.value) {
    if (
      action.componentId === hoveredComponentId.value &&
      action.state === ActionState.Failed
    ) {
      highlightedIds.add(action.id);
    }
  }
  return highlightedIds;
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
const componentListQuery = useQuery<ComponentInList[]>({
  queryKey: componentQueryKey,
  enabled: ctx.queriesEnabled,
  queryFn: async () => {
    const arg = selectedViewId.value
      ? args<Listable>(EntityKind.ViewComponentList, selectedViewId.value)
      : args<Listable>(EntityKind.ComponentList);
    const list = await bifrostList<ComponentInList[]>(arg);
    return list ?? [];
  },
});
const placeholderSearchText = computed(
  () =>
    `Search across ${componentListQuery.data.value?.length ?? 0} Components`,
);
const componentList = computed(() => componentListQuery.data.value ?? []);
const pinnedComponent = computed(() =>
  componentList.value.find((c) => c.id === pinnedComponentId.value),
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

const resourceCount = computed(
  () => componentList.value.filter((c) => c.hasResource).length ?? 0,
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
  // NOTE: We also do this to get a new array, so that later sort() calls do not mutate the
  // filteredComponents array.
  // NOTE: we reverse this because we want descending order, but sortBy only does ascending.
  let components = _.reverse(_.sortBy(filteredComponents.value, (c) => c.id));

  // Second, perform any secondary sorts, if applicable. This relies on the fact that the
  // components are already sorted.
  if (sortBySelection.value === SortByCriteria.OldestToLatest) {
    components.sort((a, b) => -b.id.localeCompare(a.id));
  } else if (sortBySelection.value === SortByCriteria.FailingActions) {
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
      Unconnected: [],
    };
    for (const component of components) {
      // Skip the pinned component itself - it's already in the "Pinned" group
      if (component.id === pinnedComponent.value.id) {
        continue;
      }

      let hasConnection = false;

      // This is subtle, but we do not use "else-if". This should have no opinion on whether or not
      // something can be both an input or an output.
      if (pinnedComponentConnectionSets.value.incoming.has(component.id)) {
        groups["Incoming connections"] ??= [];
        groups["Incoming connections"].push(component);
        hasConnection = true;
      }
      if (pinnedComponentConnectionSets.value.outgoing.has(component.id)) {
        groups["Outgoing connections"] ??= [];
        groups["Outgoing connections"].push(component);
        hasConnection = true;
      }

      // Component is neither incoming nor outgoing, so it's unconnected
      if (!hasConnection) {
        groups.Unconnected ??= [];
        groups.Unconnected.push(component);
      }
    }
  } else if (groupBySelection.value === "Diff Status") {
    groups = {
      "With Diffs": [],
      "No Diffs": [],
    };
    for (const component of components) {
      const title = component.hasDiff ? "No Diffs" : "With Diffs";
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
      const title = upgradeableComponentIds.value.has(component.id)
        ? "Upgradeable"
        : "Up to date";
      groups[title]?.push(component);
    }
  } else if (groupBySelection.value === "Resource") {
    groups = {
      "Has Resource": [],
      "No Resource": [],
    };
    for (const component of components) {
      const title = component.hasResource ? "Has Resource" : "No Resource";
      groups[title]?.push(component);
    }
  } else if (groupBySelection.value === "Schema Name") {
    const unsortedGroups: Record<string, ComponentInList[]> = {};
    for (const component of components ?? []) {
      const schemaName = component.schemaName;
      (unsortedGroups[schemaName] ??= []).push(component);
    }

    groups = Object.fromEntries(
      Object.entries(unsortedGroups).sort(([a], [b]) => a.localeCompare(b)),
    );
  } else {
    groups[""] = components;
  }

  return groups;
});

// ================================================================================================
// THE SEARCH BAR AND FILTERING
const searchString = ref<string>("");

watch(searchString, () => {
  // Update the query of the route (allowing for URL links) when the group by selection change.
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };

  if (!searchString.value) {
    delete query.searchQuery;
  } else {
    query.searchQuery = searchString.value;
  }

  router.replace({
    query,
  });
});

const filteredComponentsQueryKey = key(EntityKind.ComponentSearch);

/**
 * Components list filtered by all search terms.
 */
const filteredComponentsQuery = useQuery({
  queryKey: filteredComponentsQueryKey,
  queryFn: async () => {
    const searchTerms = parseSearch(searchString.value);

    // Filter components based on the parsed (debounced) search string.
    const workspaceId = ctx.workspacePk.value;
    const changeSetId = ctx.changeSetId.value;
    let components = componentList.value;
    if (searchTerms) {
      components = await search(components, searchTerms);
    }
    return components;

    /** Recursively apply the search query, one term at a time, honoring boolean operators */
    async function search(
      components: ComponentInList[],
      term: SearchTerms,
    ): Promise<ComponentInList[]> {
      // NOTE: this does an exhaustiveness check
      switch (term.op) {
        case "not": {
          // Find the matches, then pick everything else
          const removeComponents = new Set(
            (await search(components, term.condition)).map((c) => c.id),
          );
          return components.filter((c) => !removeComponents.has(c.id));
        }
        case "and": {
          // Just narrow down the results by applying each condition.
          for (const condition of term.conditions) {
            components = await search(components, condition);
          }
          return components;
        }
        case "or": {
          // Figure out which things match; but maintain the order of the individual searches
          const results = new Set<ComponentInList>();
          for (const condition of term.conditions) {
            // Add results in the order they were defined (unless they are duplicates)
            for (const component of await search(components, condition)) {
              results.add(component);
            }
          }
          return Array.from(results);
        }
        case "exact": {
          // Make sure the term is an exact match for name/schemaName/schemaCategory/id
          // TODO AWS::EC2::Instance vs. Instance: should both work?
          // TODO support *
          return components.filter(
            (c) =>
              c.name.localeCompare(term.value) === 0 ||
              c.schemaCategory.localeCompare(term.value) === 0 ||
              c.schemaName.localeCompare(term.value) === 0 ||
              c.id.localeCompare(term.value) === 0,
          );
        }
        case "startsWith": {
          // Make sure the term is an exact match for name/schemaName/schemaCategory/id
          // TODO AWS::EC2::Instance vs. Instance: should both work?
          // TODO support *
          const value = term.value.toLowerCase();
          return components.filter(
            (c) =>
              c.name.toLowerCase().startsWith(value) ||
              c.schemaCategory.toLowerCase().startsWith(value) ||
              c.schemaName.toLowerCase().startsWith(value) ||
              c.id.toLowerCase().startsWith(value),
          );
        }
        case "fuzzy": {
          // Regular fuzzy search across all fields
          const fzf = new Fzf(components, {
            casing: "case-insensitive",
            selector: (c) =>
              `${c.name} ${c.schemaCategory} ${c.schemaName} ${c.id}`,
          });
          return fzf.find(term.value).map((fz) => fz.item);
        }
        case "attr": {
          // Query to find the component IDs matching this attr, then use that to narrow the components
          const startTerms = term.startsWith.map((value) => ({
            key: term.key,
            value,
            op: "startsWith" as const,
          }));
          const exactTerms = term.exact.map((value) => ({
            key: term.key,
            value,
            op: "exact" as const,
          }));

          // If we get a key with no value (key:), we push in a single empty string, which will match
          // all components with that key set to anything
          if (exactTerms.length === 0 && startTerms.length === 0) {
            startTerms.push({
              key: term.key,
              value: "",
              op: "startsWith" as const,
            });
          }

          const componentIds = new Set(
            await bifrostQueryAttributes({
              workspaceId,
              changeSetId,
              terms: [...startTerms, ...exactTerms],
            }),
          );
          return components.filter((c) => componentIds.has(c.id));
        }
        default:
          return assertUnreachable(term);
      }
    }
  },
});

// Make filteredComponentsQuery reactive to its inputs (componentList)
// TODO filteredComponents needs to be a reactive thing, but queryFns aren't reactive.
const queryClient = useQueryClient();
watch(
  [componentList, searchString],
  () => {
    queryClient.invalidateQueries({
      queryKey: filteredComponentsQueryKey.value,
    });
  },
  // Invalidating the query when loading in ensures we rerun the empty filter (show everything) when
  // coming back to this page without a cached search string
  { immediate: true },
);

const filteredComponents = computed(
  () => filteredComponentsQuery.data.value ?? [],
);

function assertUnreachable(_: never): never {
  throw new Error("Didn't expect to get here");
}

// Clear the selection when the filter changes
// TODO leave the selection as long as it is still one of the filtered components?
watch(filteredComponents, () => {
  mapRef.value?.deselect();
  clearSelection();
});

// Watch for changes to fuzzySearchString and update the debounced version
watch(searchString, (newValue, oldValue) => {
  if (oldValue === "" && newValue === null) {
    // this is not a real change in the search string!
    return;
  }
  mapRef.value?.deselect();
  clearSelection();
});

// ================================================================================================
// FOCUSING, TABBING, ETC.
const focusedComponentIdx = ref<number | undefined>(-1);
const selectedComponentIndexes = reactive<Set<number>>(new Set());
const selectedComponents = computed(
  () => exploreGridRef.value?.selectedComponents ?? [],
);
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

  if (focusedComponentIdx.value > allVisibleComponents.value.length - 1) {
    if (wrap) {
      focusedComponentIdx.value = -1;
    } else {
      focusedComponentIdx.value = allVisibleComponents.value.length - 1;
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
      desiredIdx = allVisibleComponents.value.length - 1;
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
  focusedComponentIdx.value !== undefined && focusedComponentIdx.value > -1
    ? exploreGridRef.value?.getGridComponentRefByIndex(
        focusedComponentIdx.value,
      )
    : undefined,
);

const selectionComponentsForAction = computed(() => {
  if (selectedComponents.value.length > 0) return selectedComponents.value;
  else if (focusedComponent.value) return [focusedComponent.value];
  else return undefined;
});
const selectionComponentsForActionIds = computed(() => {
  if (selectionComponentsForAction.value) {
    return selectionComponentsForAction.value.map((component) => component.id);
  } else return undefined;
});
const allSelectedComponentsAreUpgradeable = computed(() => {
  if (!selectionComponentsForActionIds.value) return false;

  const notUpgradeable = selectionComponentsForActionIds.value.find(
    (componentId) => !upgradeableComponentIds.value.has(componentId),
  );
  return notUpgradeable === undefined;
});
const allSelectedComponentsAreRestorable = computed(() => {
  if (!selectionComponentsForAction.value) return false;

  const notRestorable = selectionComponentsForAction.value.find(
    (component) => !component.toDelete,
  );
  return notRestorable === undefined;
});

const fixContextMenu = () => {
  // If we focus on the pinned component, do not bring up the context menu.
  if (
    focusedGridComponentRef.value &&
    selectionComponentsForAction.value &&
    !focusedComponentIsPinned.value
  ) {
    componentContextMenuRef.value?.open(
      focusedGridComponentRef.value,
      selectionComponentsForAction.value,
    );
  }
};
const unfocus = () => {
  focusedComponentIdx.value = undefined; // don't focus the search bar on unfocus
  componentContextMenuRef.value?.close();
};
const clearSelection = () => {
  selectedComponentIndexes.clear();
  unfocus();
};

const selectComponent = (componentIdx: number) => {
  selectedComponentIndexes.add(componentIdx);
  focusedComponentIdx.value = componentIdx;
  fixContextMenu();
};
const deselectComponent = (componentIdx: number) => {
  selectedComponentIndexes.delete(componentIdx);
  if (componentIdx === focusedComponentIdx.value) {
    if (selectedComponentIndexes.size === 0) {
      clearSelection();
    } else {
      focusedComponentIdx.value = [...selectedComponentIndexes].pop();
    }
  }
  fixContextMenu();
};
const toggleComponentSelection = (componentIdx: number) => {
  if (isComponentSelected(componentIdx)) {
    deselectComponent(componentIdx);
  } else {
    selectComponent(componentIdx);
  }
};
const isComponentSelected = (componentIdx: number) =>
  selectedComponentIndexes.has(componentIdx);

// ================================================================================================
// CLICKING AND NAVIGATION
const componentClicked = (
  e: MouseEvent,
  componentId: ComponentId,
  componentIdx: number,
) => {
  e.preventDefault();
  if (e.shiftKey) {
    // multi select time!
    toggleComponentSelection(componentIdx);
    return;
  }

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
    query: preserveExploreState(
      router.currentRoute.value?.query as SelectionsInQueryString,
    ),
  });
};

watch([focusedComponent], () => {
  if (!focusedComponent.value) return;

  fixContextMenu();
});

// ================================================================================================
// KEYBOARD NAVIGATION
const searchRef = ref<InstanceType<typeof VormInput>>();
const mountEmitters = () => {
  removeEmitters();
  keyEmitter.on("a", onA);
  keyEmitter.on("c", onC);
  keyEmitter.on("k", onK);
  keyEmitter.on("n", onN);
  keyEmitter.on("e", onE);
  keyEmitter.on("d", onD);
  keyEmitter.on("u", onU);
  keyEmitter.on("r", onR);
  keyEmitter.on("p", onP);
  keyEmitter.on("ArrowRight", onArrow);
  keyEmitter.on("ArrowLeft", onArrow);
  keyEmitter.on("Enter", onEnter);
  keyEmitter.on("Tab", onTab);
  keyEmitter.on("Escape", onEscape);
  keyEmitter.on("Backspace", onBackspace);
  keyEmitter.on("Delete", onBackspace);
  keyEmitter.on("/", openShortcutModal);
  keyEmitter.on("?", openShortcutModal);
  keyEmitter.on("m", onM);
  windowResizeEmitter.on("resize", onResize);
};
const removeEmitters = () => {
  keyEmitter.off("a", onA);
  keyEmitter.off("c", onC);
  keyEmitter.off("k", onK);
  keyEmitter.off("n", onN);
  keyEmitter.off("e", onE);
  keyEmitter.off("d", onD);
  keyEmitter.off("u", onU);
  keyEmitter.off("r", onR);
  keyEmitter.off("p", onP);
  keyEmitter.off("ArrowRight", onArrow);
  keyEmitter.off("ArrowLeft", onArrow);
  keyEmitter.off("Enter", onEnter);
  keyEmitter.off("Tab", onTab);
  keyEmitter.off("Escape", onEscape);
  keyEmitter.off("Backspace", onBackspace);
  keyEmitter.off("Delete", onBackspace);
  keyEmitter.off("/", openShortcutModal);
  keyEmitter.off("?", openShortcutModal);
  keyEmitter.off("m", onM);
  windowResizeEmitter.off("resize", onResize);
};

const onA = (e: KeyDetails["a"]) => {
  e.preventDefault();
  if (e.metaKey || e.ctrlKey) {
    const components = allVisibleComponents.value;
    [...components.keys()].forEach((index) => {
      const component = componentList.value[index];
      if (component) {
        selectComponent(index);
      }
    });
  }
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
    clearSelection();
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
    if (!selectionComponentsForAction.value) return;

    componentContextMenuRef.value?.componentsStartErase(
      selectionComponentsForAction.value,
    );
  } else {
    mapRef.value?.onE(e);
  }
};
const onD = (e: KeyDetails["d"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!selectionComponentsForActionIds.value) return;
    componentContextMenuRef.value?.componentsDuplicate(
      selectionComponentsForActionIds.value,
    );
  } else {
    mapRef.value?.onD(e);
  }
};
const onP = (e: KeyDetails["p"]) => {
  // You can only pin one component at a time!
  if (selectedComponentIndexes.size > 1) return;

  e.preventDefault();
  if (showGrid.value) {
    if (!focusedComponent.value || selectedComponents.value.length > 1) return;

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
    if (!selectionComponentsForActionIds.value) return;

    if (allSelectedComponentsAreUpgradeable.value) {
      componentContextMenuRef.value?.componentsUpgrade(
        selectionComponentsForActionIds.value,
      );
    }
  } else {
    mapRef.value?.onU(e);
  }
};
const onBackspace = (e: KeyDetails["Backspace"] | KeyDetails["Delete"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!selectionComponentsForAction.value) return;
    componentContextMenuRef.value?.componentsStartDelete(
      selectionComponentsForAction.value,
    );
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
    if (!selectionComponentsForActionIds.value) return;
    if (allSelectedComponentsAreRestorable.value) {
      componentContextMenuRef.value?.componentsRestore(
        selectionComponentsForActionIds.value,
      );
    }
  } else {
    mapRef.value?.onR(e);
  }
};
const onM = (e: KeyDetails["m"]) => {
  e.preventDefault();
  if (showGrid.value) {
    // Do nothing in grid mode
    return;
  }
  mapRef.value?.onM(e);
};
const onEscape = () => {
  if (isThereAModalOpen.value) return;

  if (showGrid.value) {
    searchRef.value?.blur();
    clearSelection();
  } else {
    mapRef.value?.onEscape();
  }
};

const onTab = (e: KeyDetails["Tab"], blurSearch = false) => {
  e.preventDefault();
  if (!showGrid.value) return; // no tab behavior on the map yet
  if (isThereAModalOpen.value) return; // no tab behavior when a modal is open

  selectedComponentIndexes.clear();
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
  if (selectedComponentIndexes.size > 1) {
    // TODO(Wendy) - for now, this does nothing.
    return;
  }

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

const onArrow = () => {
  componentContextMenuRef.value?.focusFirstItem(true);
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
  if (elementIsScrolledIntoView(el)) {
    fixContextMenu();
  } else {
    unfocus();
    if (selectedComponentIndexes.size > 0) {
      // we are in a situation where the menu is not showing for a multi select!
      // try to find the best component to put it on
      const selectedIndexesArray = [...selectedComponentIndexes];
      for (const selectedIndex of selectedIndexesArray) {
        const el =
          exploreGridRef.value?.getGridComponentRefByIndex(selectedIndex)?.$el;
        if (elementIsScrolledIntoView(el)) {
          focusedComponentIdx.value = selectedIndex;
          break;
        }
      }
      if (focusedComponentIdx.value === undefined) {
        focusedComponentIdx.value = selectedIndexesArray.pop();
      }
    }
  }
};
const onResize = () => {
  clearSelection();
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

    clearSelection();
  }
};

// ================================================================================================
// MOUNTING AND URL QUERY HANDLING
const setSelectionsFromQuery = () => {
  const query: SelectionsInQueryString = router.currentRoute.value?.query;

  if (query.searchQuery !== undefined) {
    searchString.value = query.searchQuery;
  }

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
    case "schemaname":
      groupBySelection.value = GroupByCriteria.SchemaName;
      break;
    case "resource":
      groupBySelection.value = GroupByCriteria.Resource;
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
  if (focusedComponent.value && selectedComponentIndexes.size < 2) {
    componentNavigate(focusedComponent.value.id);
  }
};

// ================================================================================================
// MODAL REFS
const isThereAModalOpen = computed(
  () =>
    shortcutModalRef.value?.isOpen ||
    addComponentModalRef.value?.isOpen ||
    addViewModalRef.value?.isOpen ||
    editViewModalRef.value?.isOpen,
);

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
  | "upgradeable"
  | "schemaname"
  | "resource";

enum GroupByCriteria {
  Diff = "Diff Status",
  Upgrade = "Upgradeable",
  Qualification = "Qualification Status",
  SchemaName = "Schema Name",
  Resource = "Resource",
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
  { value: GroupByCriteria.SchemaName, label: "Schema Name" },
  { value: GroupByCriteria.Resource, label: "Resource" },
];

watch([groupBySelection], () => {
  // First, make sure we clear all selections.
  clearSelection();

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
  } else if (groupBySelection.value === GroupByCriteria.SchemaName) {
    query.groupBy = "schemaname";
  } else if (groupBySelection.value === GroupByCriteria.Resource) {
    query.groupBy = "resource";
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
  OldestToLatest = "Oldest to latest",
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
  // NOTE(victor, wendy): We use this option for testing how things react to sorting,
  // so we can keep it around even though it's not meant for release
  // { value: SortByCriteria.OldestToLatest, label: "Oldest to latest" },
  { value: SortByCriteria.FailingActions, label: "Failing actions" },
  { value: SortByCriteria.RunningActions, label: "Running actions" },
];

watch([sortBySelection], () => {
  // First, make sure we clear all selections.
  clearSelection();

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
