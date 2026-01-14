<template>
  <section :class="clsx('grid h-full', showGrid ? 'explore' : 'map')">
    <!-- Left column -->
    <!-- 12 pixel padding to align with the SI logo -->
    <div
      data-testid="left-column-new-hotness-explore"
      class="main pt-sm flex flex-col gap-sm items-stretch [&>div]:mx-[12px]"
    >
      <!-- Socket connections banner -->
      <div
        v-if="hasSocketConnections && !showSkeleton"
        :class="
          clsx(
            'flex flex-row items-center gap-xs px-sm py-xs border my-sm',
            themeClasses(
              'bg-neutral-200 border-neutral-400 text-neutral-900',
              'bg-neutral-700 border-neutral-600 text-neutral-100',
            ),
          )
        "
      >
        <TruncateWithTooltip class="py-2xs text-sm flex-1">
          A faster, smarter experience is here. Some component settings may be incompatible.
        </TruncateWithTooltip>
        <NewButton size="sm" tone="action" label="Learn more" @click="openWorkspaceMigrationDocumentation" />
      </div>

      <!-- Search and filters -->
      <ExploreSearchBarSkeleton v-if="showSkeleton" />
      <template v-else>
        <div class="flex-none flex flex-row items-start gap-xs">
          <DropdownMenuButton
            ref="viewsDropdownRef"
            class="rounded min-w-[128px] h-[38px] pl-xs"
            :options="filteredViewListOptions"
            :search="viewListOptions.length > DEFAULT_DROPDOWN_SEARCH_THRESHOLD"
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
          <div class="grow relative">
            <InstructiveVormInput
              :class="clsx('rounded cursor-text')"
              :activeClasses="themeClasses('border-action-500', 'border-action-300')"
              :inactiveClasses="
                themeClasses('border-neutral-400 hover:border-black', 'border-neutral-600 hover:border-white')
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
                      showSearchFooter = true;
                    }
                  "
                  @blur="
                    () => {
                      slotProps.blur();
                      showSearchFooter = false;
                    }
                  "
                  @keydown.tab="(e: KeyboardEvent) => onTab(e, true)"
                  @keydown.esc="onEscape"
                />
              </template>
            </InstructiveVormInput>

            <!-- Search footer - absolute positioned overlay -->
            <Transition
              enterActiveClass="transition duration-200 ease-out"
              enterFromClass="opacity-0 translate-y-1"
              enterToClass="opacity-100 translate-y-0"
              leaveActiveClass="transition duration-150 ease-in"
              leaveFromClass="opacity-100 translate-y-0"
              leaveToClass="opacity-0 translate-y-1"
            >
              <div
                v-if="showSearchFooter"
                :class="
                  clsx(
                    'absolute top-full left-0 right-0 z-50',
                    'flex items-center gap-1 px-sm py-2xs text-xs',
                    'border border-t-0 rounded-b-sm',
                    themeClasses(
                      'bg-neutral-50 border-neutral-300 text-neutral-600',
                      'bg-neutral-800 border-neutral-600 text-neutral-300',
                    ),
                  )
                "
              >
                <a
                  href="https://docs.systeminit.com/reference/search"
                  target="_blank"
                  rel="noopener noreferrer"
                  :class="
                    clsx(
                      'cursor-pointer underline',
                      themeClasses('text-neutral-600 hover:text-black', 'text-neutral-300 hover:text-white'),
                    )
                  "
                >
                  Learn more about our search query syntax
                </a>
              </div>
            </Transition>
          </div>
        </div>
        <div class="flex-none flex flex-row flex-wrap items-center gap-xs">
          <TabGroupToggle ref="groupRef" :aOrB="urlGridOrMap === 'grid'" @toggle="storeViewMode">
            <template #a="{ selected, toggle }">
              <ExploreModeTile icon="grid" label="Grid" :selected="selected" @toggle="toggle" />
            </template>
            <template #b="{ selected, toggle }">
              <ExploreModeTile icon="map" label="Map" :selected="selected" @toggle="toggle" />
            </template>
          </TabGroupToggle>
          <template v-if="!showGrid">
            <button
              v-if="!ctx.onHead.value"
              :class="
                clsx(
                  'flex flex-row gap-xs items-center border rounded-sm',
                  'p-2xs mb-[-1px] h-7',
                  'font-mono text-[13px] text-left truncate relative',
                  themeClasses(
                    'border-neutral-400 hover:border-action-500',
                    'border-neutral-600 hover:border-action-300',
                  ),
                  queryOnlyDiff
                    ? themeClasses('bg-action-200', 'bg-action-900')
                    : themeClasses('bg-neutral-100', 'bg-neutral-900'),
                )
              "
              @click="toggleOnlyDiff"
            >
              <Icon name="tilde-circle" :class="themeClasses('text-warning-500', 'text-warning-300')" size="xs" />
              <span>See only diffs</span>
            </button>
            <button
              v-if="mapRef?.selectedComponents.size || 0 > 0"
              :class="
                clsx(
                  'flex flex-row gap-xs items-center border rounded-sm',
                  'p-2xs mb-[-1px] h-7',
                  'font-mono text-[13px] text-left truncate relative',
                  themeClasses(
                    'border-neutral-400 hover:border-action-500',
                    'border-neutral-600 hover:border-action-300',
                  ),
                  queryHideSubscriptions
                    ? themeClasses('bg-action-200', 'bg-action-900')
                    : themeClasses('bg-neutral-100', 'bg-neutral-900'),
                )
              "
              @click="toggleHide"
            >
              <Icon name="hide" size="xs" />
              <span>Hide unconnected components</span>
            </button>
          </template>
          <div v-if="showGrid" class="ml-auto flex flex-row flex-wrap gap-xs">
            <DefaultSubscriptionsButton
              :selected="gridMode.mode === 'defaultSubscriptions'"
              @click="clickDefaultSubscriptionsButton"
            />
            <DropdownMenuButton
              class="rounded-sm"
              hoverBorder
              :options="groupByDropDownOptions"
              :modelValue="gridMode.mode === 'groupBy' ? gridMode.criteria : ''"
              placeholder="Group by"
              minWidthToAnchor
              checkable
              alwaysShowPlaceholder
              highlightWhenModelValue
              @update:modelValue="updateGroupBy"
            >
              <template #beforeOptions>
                <DropdownMenuItem
                  label="None"
                  value="''"
                  checkable
                  :checked="gridMode.mode !== 'groupBy'"
                  @select="clearGroupBy"
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
              class="rounded-sm"
              hoverBorder
              :options="sortByDropDownOptions"
              :modelValue="sortBySelection"
              placeholder="Sort by"
              minWidthToAnchor
              checkable
              alwaysShowPlaceholder
              :highlightWhenModelValue="sortBySelection !== SortByCriteria.LatestToOldest"
              @update:modelValue="(val) => (sortBySelection = sortByFromString(val))"
            />
          </div>
        </div>
      </template>

      <template v-if="showGrid">
        <ExploreGridSkeleton v-if="showSkeleton" />
        <template v-else>
          <DefaultSubscriptionsEmptyState
            v-if="gridMode.mode === 'defaultSubscriptions' && defaultSubscriptions.defaultSubscriptions.size === 0"
          />
          <div
            v-else-if="componentList.length === 0 && componentListQuery.isSuccess.value"
            class="flex-1 gap-sm scrollable flex flex-col items-center"
          >
            <div class="grow flex items-center justify-center">
              <EmptyState icon="logo-si" iconSize="lg" iconNoBg text="Your data will be shown here">
                <template #secondary>
                  <div v-if="!hasUsedAiAgent" class="flex flex-row text-neutral-400">
                    Finish setting up your AI agent to see the platform in action.
                  </div>

                  <div class="flex flex-row gap-sm">
                    <NewButton label="Restart Workspace Setup" tone="action" @click="ctx.reopenOnboarding" />
                    <NewButton
                      label="How-to guide"
                      tone="neutral"
                      href="https://docs.systeminit.com/how-tos/"
                      target="_blank"
                    />
                    <NewButton
                      label="API guidelines"
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
                  'shrink-0 flex flex-col items-center py-md px-lg w-full max-w-6xl mb-sm gap-3 border text-center rounded-sm',
                  themeClasses('bg-neutral-200 border-neutral-400', 'bg-neutral-800 border-neutral-600'),
                )
              "
            >
              <span class="font-bold">Explore other ways to get started</span>
              <span> Prefer a different starting point? Here are a few other ways to jump in and explore: </span>
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
                <a href="https://www.systeminit.com/?modal=demo" target="_blank"> Schedule a demo </a>
                <a href="https://discord.gg/system-init" target="_blank"> Join the community </a>
                <a :href="`https://auth.systeminit.com/workspace/${ctx?.workspacePk.value}`" target="_blank">
                  Invite a teammate
                </a>
              </div>
            </div>
          </div>
          <div
            v-else
            ref="scrollRef"
            data-testid="tile-container"
            :class="clsx('grow', bulkEditing ? 'min-h-0' : 'scrollable')"
            :style="!bulkEditing && 'overflow-anchor: none;'"
            @scroll="onScroll"
            @scrollend="onScrollEnd"
          >
            <WelcomeBanner v-if="ctx.onHead.value" class="mb-sm" />
            <ExploreGrid
              ref="exploreGridRef"
              :components="sortedAndGroupedComponents"
              :gridRows="gridRows"
              :scrollRef="scrollRef"
              :bulkEditing="bulkEditing"
              @componentNavigate="componentNavigate"
              @resetFilter="resetFilter"
              @bulkDone="bulkDone"
              @childClicked="componentClicked"
              @childSelect="
                (idx: number, event?: MouseEvent) => selectComponent(idx, event)
              "
              @childDeselect="(idx: number) => deselectComponent(idx)"
              @childHover="
                (componentId) => {
                  if (componentsHaveActionsWithState.failed.has(componentId)) {
                    hoveredComponentId = componentId;
                  }
                }
              "
              @childUnhover="() => (hoveredComponentId = undefined)"
              @unpin="() => (gridMode = { mode: 'default', label: '' })"
              @collapse="collapse"
              @selectAllInSection="selectAllInSection"
              @deselectAllInSection="deselectAllInSection"
            />
          </div>
          <footer
            id="footer"
            :class="
              clsx(
                'flex-none h-12 px-xs border-t flex flex-row items-center gap-sm',
                themeClasses('bg-neutral-100 border-neutral-400', 'bg-neutral-800 border-neutral-600'),
              )
            "
          >
            <!-- footer, for bulk editing we teleport contents in here -->
            <template v-if="!bulkEditing">
              <TruncateWithTooltip
                v-if="!ctx.onHead.value && ctx.changeSet.value"
                :class="
                  clsx(
                    'h-[30px] leading-[24px]',
                    'text-sm font-mono py-3xs px-2xs text-center',
                    themeClasses('text-success-700', 'text-success-300'),
                  )
                "
              >
                You are in a simulated change set:
                {{ ctx.changeSet.value.name }}
              </TruncateWithTooltip>
              <!-- <NewButton
                label="See keyboard shortcuts"
                pill="?"
                tone="neutral"
                @click="openShortcutModal"
              /> -->
              <NewButton
                class="ml-auto"
                label="Add a component"
                pill="N"
                tone="action"
                @click="openAddComponentModal"
              />
            </template>
          </footer>
        </template>
      </template>
      <MapComponent
        v-else
        ref="mapRef"
        :active="!showGrid"
        :components="filteredComponents"
        :componentsWithFailedActions="componentsHaveActionsWithState.failed"
        @deselect="onMapDeselect"
        @selectedComponents="onMapSelectedComponentsChange"
        @help="openShortcutModal"
      />
    </div>
    <!-- Right column -->
    <div
      data-testid="right-column-new-hotness-explore"
      :class="
        clsx(
          'right flex flex-col border-l',
          themeClasses('bg-neutral-100 border-neutral-400', 'bg-neutral-800 border-neutral-600'),
        )
      "
    >
      <!-- Skeleton -->
      <ExploreRightColumnSkeleton v-if="showSkeleton" />
      <template v-else>
        <div class="grow grid grid-rows-subgrid min-h-0" :style="collapsingStyles">
          <CollapsingGridItem ref="actionsRef">
            <template #header><span class="text-sm">Actions</span></template>
            <template #headerIconsRight>
              <PillCounter :count="actionViewList.length" class="text-sm" />
            </template>
            <ActionQueueList :actionViewList="actionViewList" :highlightedActionIds="highlightedActionIds" />
          </CollapsingGridItem>
          <CollapsingGridItem ref="historyRef" disableScroll>
            <template #header><span class="text-sm">Recent function runs</span></template>
            <FuncRunList :limit="25" />
          </CollapsingGridItem>
          <CollapsingGridItem v-if="ffStore.SHOW_POLICIES" ref="policyRef" disableScroll>
            <template #header><span class="text-sm">Policy history</span></template>
            <PolicyList
              :policies="policyReports"
              :page="page"
              :maxPages="maxPages"
              @pageBack="pageBack"
              @pageForward="pageForward"
              @select="(p) => navigateToPolicy(p)"
            />
          </CollapsingGridItem>
        </div>
        <div
          :class="
            clsx(
              'flex-none border-t flex flex-col gap-xs p-xs min-h-[48px]',
              themeClasses('border-neutral-400', 'border-neutral-600'),
            )
          "
        >
          <!-- Top row with counts -->
          <div class="flex flex-row items-center gap-xs">
            <TextPill v-tooltip="componentCountTooltip" class="flex-none rounded p-xs text-sm" variant="key2">
              Total: {{ componentList.length }}
            </TextPill>
            <TextPill
              v-if="resourceCount > 0"
              v-tooltip="resourceCountTooltip"
              :class="
                clsx(
                  'flex-none flex flex-row items-center gap-2xs rounded p-xs text-sm',
                  themeClasses(
                    'border-success-400 bg-success-100 text-black',
                    'border-success-800 bg-success-900 text-white',
                  ),
                )
              "
            >
              <Icon :class="themeClasses('text-success-600', 'text-success-400')" name="check-hex-outline" size="xs" />
              {{ resourceCount }}
            </TextPill>
          </div>
          <!-- Status message row (appears below when needed) -->
          <RealtimeStatusPageState />
        </div>
      </template>
    </div>

    <!-- MODALS -->
    <ShortcutModal ref="shortcutModalRef" />
    <AddComponentModal ref="addComponentModalRef" />
    <AddViewModal ref="addViewModalRef" :views="viewListQuery.data.value ?? []" />
    <!-- For the edit view modals, upon delete, change back to "All Views" -->
    <EditViewModal
      ref="editViewModalRef"
      :views="viewListQuery.data.value ?? []"
      @deleted="() => (selectedViewId = '')"
    />
    <ComponentContextMenu
      ref="componentContextMenuRef"
      onGrid
      enableKeyboardControls
      :viewListOptions="viewListOptions"
      @clearSelected="clearSelection"
      @edit="navigateToFocusedComponent"
      @pin="
        (c) => {
          gridMode = { mode: 'pinned', label: '', componentId: c };
        }
      "
      @bulk="startBulkEdit"
      @finishAction="onMenuFinishAction"
    />
  </section>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, nextTick, onBeforeUnmount, onMounted, provide, reactive, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  NewButton,
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
  PillCounter,
  TextPill,
  themeClasses,
  TruncateWithTooltip,
  VormInput,
  DEFAULT_DROPDOWN_SEARCH_THRESHOLD,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { tw } from "@si/vue-lib";
import { useToast } from "vue-toastification";
import { bifrost, bifrostList, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  ComponentInList,
  EntityKind,
  View,
  DefaultSubscription,
  DefaultSubscriptions,
} from "@/workers/types/entity_kind_types";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import { Listable } from "@/workers/types/dbinterface";
import { elementIsScrolledIntoView } from "@/newhotness/logic_composables/dom_funcs";
import { ActionState } from "@/api/sdf/dal/action";
import ExploreSearchBarSkeleton from "@/newhotness/skeletons/ExploreSearchBarSkeleton.vue";
import ExploreGridSkeleton from "@/newhotness/skeletons/ExploreGridSkeleton.vue";
import ExploreRightColumnSkeleton from "@/newhotness/skeletons/ExploreRightColumnSkeleton.vue";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import WelcomeBanner from "@/newhotness/WelcomeBanner.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import MapComponent from "./Map.vue";
import { collapsingGridStyles, openWorkspaceMigrationDocumentation } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import { getQualificationStatus } from "./ComponentTileQualificationStatus.vue";
import FuncRunList from "./FuncRunList.vue";
import { ComponentsHaveActionsWithState, ExploreContext, GridMode } from "./types";
import {
  KeyDetails,
  keyEmitter,
  MouseDetails,
  mouseEmitter,
  windowResizeEmitter,
  windowWidthReactive,
} from "./logic_composables/emitters";
import TabGroupToggle from "./layout_components/TabGroupToggle.vue";
import { SelectionsInQueryString } from "./Workspace.vue";
import AddComponentModal from "./AddComponentModal.vue";
import DefaultSubscriptionsButton from "./DefaultSubscriptionsButton.vue";
import DefaultSubscriptionsEmptyState from "./layout_components/DefaultSubscriptionsEmptyState.vue";
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
import { useComponentSearch } from "./logic_composables/search";
import { routes, useApi } from "./api_composables";
import { ExploreGridRowData } from "./explore_grid/ExploreGridRow.vue";
import { useDefaultSubscription } from "./logic_composables/default_subscriptions";
import { useContext } from "./logic_composables/context";
import { generateMockActions } from "./logic_composables/mock_data";
import PolicyList from "./layout_components/PolicyList.vue";
import { Policy, usePolicy } from "./logic_composables/policy";

const router = useRouter();
const route = useRoute();
const ctx = useContext();

const hasUsedAiAgent = computed(() => ctx.userWorkspaceFlags.value.executedAgent ?? false);

const key = useMakeKey();
const args = useMakeArgs();

const VIEW_MODE_LOCAL_STORAGE_KEY = "newhotness-view-mode";
const viewModeStorageKey = () => `${VIEW_MODE_LOCAL_STORAGE_KEY}: ${ctx.changeSetId}`;
const storeViewMode = () => {
  if (!groupRef.value) return;

  const key = viewModeStorageKey();

  if (groupRef.value.isB) {
    localStorage.setItem(key, "grid");
    fixContextMenu();
  } else {
    localStorage.setItem(key, "map");
    componentContextMenuRef.value?.close();
  }
};

const FILTER_AND_GROUP_STORAGE_KEY = "newhotness-filter-and-grouping";
const filterAndGroupStorageKey = () => `${FILTER_AND_GROUP_STORAGE_KEY}: ${ctx.changeSetId.value}`;
const storeFilterAndGroup = (query: SelectionsInQueryString) => {
  // Do not store retainSessionState, since when this query comes we're supposed to read stored data. Skipping this would always reset the storage
  if (query.retainSessionState) {
    return;
  }
  const queryString = JSON.stringify(query);
  sessionStorage.setItem(filterAndGroupStorageKey(), queryString);
};
const retrieveFilterAndGroup = (): SelectionsInQueryString => {
  const qString = sessionStorage.getItem(filterAndGroupStorageKey());

  return qString ? (JSON.parse(qString) as SelectionsInQueryString) : {};
};

const ffStore = useFeatureFlagsStore();

const defaultSubscriptions = useDefaultSubscription();

const groupRef = ref<InstanceType<typeof TabGroupToggle>>();
const actionsRef = ref<typeof CollapsingGridItem>();
const historyRef = ref<typeof CollapsingGridItem>();
const policyRef = ref<typeof CollapsingGridItem>();
const mapRef = ref<InstanceType<typeof MapComponent>>();
const exploreGridRef = ref<InstanceType<typeof ExploreGrid>>();
const componentContextMenuRef = ref<InstanceType<typeof ComponentContextMenu>>();

const collapsingStyles = computed(() => {
  const grids = [actionsRef.value?.openState, historyRef.value?.openState];
  if (ffStore.SHOW_POLICIES) grids.push(policyRef.value?.openState);

  return collapsingGridStyles(grids);
});

const { policyReports, page, maxPages } = usePolicy();

const pageBack = () => {
  if (page.value === 1) page.value = maxPages.value;
  else page.value -= 1;
};
const pageForward = () => {
  if (page.value === maxPages.value) page.value = 1;
  else page.value += 1;
};

const navigateToPolicy = (policy: Policy) => {
  const params = {
    workspacePk: route.params.workspacePk,
    changeSetId: route.params.changeSetId,
    policyId: policy.id,
  };
  router.push({
    name: "new-hotness-policy",
    params,
    query: {
      page: page.value,
    },
  });
};

const queryOnlyDiff = computed(() => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  return query.showDiff === "1";
});

const toggleOnlyDiff = () => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  if (queryOnlyDiff.value) {
    delete query.showDiff;
  } else {
    query.showDiff = "1";
  }
  router.replace({ query });
};
const queryHideSubscriptions = computed(() => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  return query.hideSubscriptions === "1";
});

const toggleHide = () => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  if (queryHideSubscriptions.value) {
    delete query.hideSubscriptions;
  } else {
    query.hideSubscriptions = "1";
  }
  router.replace({ query });
};

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
const gridMapSwitcherValue = computed(() => groupRef.value && groupRef.value.isA);
// TODO — if youre on HEAD and you start bulk editing, create a change set right away
const bulkEditing = ref(false);

const toast = useToast();
const bulkChangeSet = useApi();
const startBulkEdit = async () => {
  componentContextMenuRef.value?.close();

  if (ctx.onHead.value) {
    const call = bulkChangeSet.endpoint<ChangeSet>(routes.CreateChangeSet);
    const { req } = await call.post({
      name: `Bulk Edit by ${ctx.user?.name}`,
    });
    if (!bulkChangeSet.ok(req)) {
      toast("Creating change set failed");
      return;
    }
    const query: SelectionsInQueryString = {
      ...router.currentRoute.value?.query,
    };
    query.b = "1";
    bulkChangeSet.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: ctx.workspacePk.value,
          changeSetId: req.data.id,
        },
        query,
      },
      req.data.id,
    );
    return;
  }
  // get rid of this old ref as its being removed from the DOM
  exploreContext.value.focusedComponentRef.value = undefined;
  bulkEditing.value = true;
};

watch(bulkEditing, () => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  if (bulkEditing.value) query.b = "1";
  else delete query.b;
  storeFilterAndGroup(query);
  router.push({ query });
});

watch(gridMapSwitcherValue, (newShowGrid) => {
  // If this is nil, groupRef is unmounted, and we don't care about the change.
  if (_.isNil(newShowGrid)) return;
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.grid;
  if (newShowGrid) query.grid = "1";
  else query.map = "1";

  storeFilterAndGroup(query);
  router.push({ query });
});

// Track hovered component for highlighting failed actions
const hoveredComponentId = ref<ComponentId | undefined>(undefined);

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
  return list.map((l) => ({ value: l.id, label: l.name })).sort((a, b) => a.label.localeCompare(b.label));
});
const viewsDropdownRef = ref<InstanceType<typeof DropdownMenuButton>>();
const filteredViewListOptions = computed(() => {
  const searchString = viewsDropdownRef.value?.searchString;

  if (!searchString || searchString === "") {
    return viewListOptions.value;
  }

  return viewListOptions.value.filter(
    (option) =>
      option.label.toLocaleLowerCase().includes(searchString) ||
      option.value.toLocaleLowerCase().includes(searchString),
  );
});

const defaultView = computed(() => viewListQuery.data.value?.find((v) => v.isDefault));
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
  if (selectedViewId.value !== "" && selectedViewId.value !== defaultView.value?.id) {
    query.viewId = selectedViewOrDefaultId.value;
  }

  storeFilterAndGroup(query);
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
    if (!viewListQuery.data.value || !viewListQuery.data.value.find((v) => v.id === selectedViewId.value)) {
      selectedViewId.value = "";
    }
  },
  { immediate: true },
);

// ================================================================================================
// ALL COMPONENTS AVAILABLE FOR USE, INCLUDING VIEWS AND PINNING
const componentListQueryKind = computed(() =>
  selectedViewId.value ? EntityKind.ViewComponentList : EntityKind.ComponentList,
);
const componentListQueryId = computed(() => (selectedViewId.value ? selectedViewId.value : ctx.workspacePk.value));
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
const placeholderSearchText = computed(() => `Search across ${componentListQuery.data.value?.length ?? 0} Components`);
const componentList = computed(() => {
  return componentListQuery.data.value ?? [];
});

const hasSocketConnections = computed(() => {
  if (!componentList.value) return false;
  return componentList.value.some((c) => c.hasSocketConnections);
});

// ================================================================================================
// GRID MODE: GROUP BY, PINNING, DEFAULT SUBSCRIPTIONS, ETC.
const gridMode = ref<GridMode>({ mode: "default", label: "" });

const groupByDropDownOptions = computed(() => {
  const baseOptions = [
    { value: "diff", label: "Diff Status" },
    { value: "qualification", label: "Qualification Status" },
    { value: "upgrade", label: "Upgradeable" },
    { value: "schemaName", label: "Schema Name" },
    { value: "resource", label: "Resource" },
  ];

  // Only show Socket Connections option if there are components with socket connections
  if (hasSocketConnections.value) {
    baseOptions.push({
      value: "incompatibleComponents",
      label: "Incompatible Components",
    });
  }

  return baseOptions;
});

const gridModeFromGroupByCriteria = (value: string): GridMode => {
  if (value === "diff")
    return {
      mode: "groupBy",
      criteria: "diff",
      label: "Diff Status",
    };
  if (value === "qualification")
    return {
      mode: "groupBy",
      criteria: "qualification",
      label: "Qualification Status",
    };
  if (value === "upgrade")
    return {
      mode: "groupBy",
      criteria: "upgrade",
      label: "Upgradeable",
    };
  if (value === "schemaName")
    return {
      mode: "groupBy",
      criteria: "schemaName",
      label: "Schema Name",
    };
  if (value === "resource")
    return {
      mode: "groupBy",
      criteria: "resource",
      label: "Resource",
    };
  if (value === "incompatibleComponents")
    return {
      mode: "groupBy",
      criteria: "incompatibleComponents",
      label: "Incompatible Components",
    };
  return {
    mode: "default",
    label: "",
  };
};

const clickDefaultSubscriptionsButton = () => {
  clearSelection();
  if (gridMode.value.mode === "defaultSubscriptions") {
    gridMode.value = {
      mode: "default",
      label: "",
    };
  } else {
    gridMode.value = {
      mode: "defaultSubscriptions",
      label: "",
    };
  }
};

const updateGroupBy = (val: string) => {
  clearSelection();
  gridMode.value = gridModeFromGroupByCriteria(val);
};
const clearGroupBy = () => {
  clearSelection();
  gridMode.value = { mode: "default", label: "" };
};

watch([hasSocketConnections, gridMode], ([newHasSocketConnections, newGridMode]) => {
  // Only new socket connections with incompatible components group by when in grid view.
  if (!showGrid.value) return;

  // If groupBy is set to IncompatibleComponents but there are no socket connections, clear it
  if (!newHasSocketConnections && newGridMode.mode === "groupBy" && newGridMode.criteria === "incompatibleComponents") {
    gridMode.value = { mode: "default", label: "" };
  }
});

watch(gridMode, (newMode, oldMode) => {
  // Ignore the grid mode in map view.
  if (!showGrid.value) {
    return;
  }

  // If we are moving in or out of pinning mode, we need to clear the selection.
  if (
    (newMode.mode === "pinned" && oldMode.mode !== "pinned") ||
    (newMode.mode !== "pinned" && oldMode.mode === "pinned")
  ) {
    clearSelection();
  }

  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.groupBy;
  delete query.pinned;
  delete query.defaultSubscriptions;

  query.grid = "1";

  if (newMode.mode === "pinned") {
    query.pinned = newMode.componentId;
  } else if (newMode.mode === "groupBy") {
    if (newMode.criteria === "diff") {
      query.groupBy = "diffstatus";
    } else if (newMode.criteria === "qualification") {
      query.groupBy = "qualificationstatus";
    } else if (newMode.criteria === "upgrade") {
      query.groupBy = "upgradeable";
    } else if (newMode.criteria === "schemaName") {
      query.groupBy = "schemaname";
    } else if (newMode.criteria === "resource") {
      query.groupBy = "resource";
    } else if (newMode.criteria === "incompatibleComponents") {
      query.groupBy = "incompatibleComponents";
    }
  } else if (newMode.mode === "defaultSubscriptions") {
    query.defaultSubscriptions = "1";
  }

  storeFilterAndGroup(query);
  router.push({
    query,
  });
});

// ================================================================================================
// SETUP THE FILTERED COMPONENTS REACTIVE AND UPGRADEABLES
const upgrade = useUpgrade();
const upgradeableComponentIds = computed(() => {
  const set: Set<ComponentId> = new Set();

  // TODO(nick): try to swap this with the component list to see if we recompute this less
  // frequently. This is not a problem today, but could be tomorrow.
  for (const component of filteredComponents.value ?? []) {
    // This needs to be split out into a variable for reactivity. Keep this here or drown in
    // sorrow and suffering. Relevant pull request: https://github.com/systeminit/si/pull/6483
    const canUpgrade = upgrade(component.schemaId, component.schemaVariantId).value;
    if (canUpgrade) {
      set.add(component.id);
    }
  }
  return set;
});

// ================================================================================================
// SKELETON BEHAVIOR

const showSkeleton = computed(
  () => componentListQuery.isLoading.value || (!_.isNil(mapRef.value) && mapRef.value.isLoading),
);

// ================================================================================================
// INSTANTIATION OF REQUIRED VARIABLES

const focusedComponentIdx = ref<number | undefined>(-1);
const selectedComponentIndexes = reactive<Set<number>>(new Set());

const setFocusedComponentIdx = (idx: number | undefined) => {
  exploreContext.value.focusedComponentRef.value = undefined;
  focusedComponentIdx.value = idx;
};

const unfocus = () => {
  setFocusedComponentIdx(undefined); // don't focus the search bar on unfocus
  componentContextMenuRef.value?.close();
};

const collapse = (title: string, collapsed: boolean) => {
  collapseTracker.value[title] = collapsed;
};

enum SortByCriteria {
  FailingActions = "Failing actions",
  RunningActions = "Running actions",
  LatestToOldest = "Latest to oldest",
  OldestToLatest = "Oldest to latest",
}
const sortBySelection = ref<SortByCriteria>(SortByCriteria.LatestToOldest);

export type GroupByUrlQuery =
  | "diffstatus"
  | "qualificationstatus"
  | "upgradeable"
  | "schemaname"
  | "resource"
  | "incompatibleComponents";

const bulkDone = () => {
  bulkEditing.value = false;
  fixContextMenu();
};

// ================================================================================================
// ACTIONS INFORMATION FOR GROUP BY
const DEBUG_USE_MOCK_ACTIONS = false;
const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () => await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionViewList = computed(() => {
  if (DEBUG_USE_MOCK_ACTIONS) {
    return generateMockActions(ctx.changeSetId.value);
  }
  return actionViewListRaw.data.value?.actions ?? [];
});

const componentsHaveActionsWithState = computed(() => {
  const results: ComponentsHaveActionsWithState = {
    failed: new Set(),
    running: new Set(),
  };
  for (const action of actionViewList.value) {
    if (!action.componentId) continue;
    if (action.state === ActionState.Failed) {
      results.failed.add(action.componentId);
    } else if (action.state === ActionState.Dispatched) {
      results.running.add(action.componentId);
    }
  }
  return results;
});

// Map component IDs to their pending action counts by type (can be multiple)
const componentsPendingActionNames = computed(() => {
  const results = new Map<ComponentId, Record<string, { count: number; hasFailed: boolean }>>();
  for (const action of actionViewList.value) {
    if (!action.componentId) continue;
    // All action states are considered "pending" and should be shown
    if (!results.has(action.componentId)) {
      results.set(action.componentId, {});
    }

    const actionCounts = results.get(action.componentId)!;

    // Group Other actions with Manual
    let actionName = action.name;
    if (actionName.toLowerCase() === "other") {
      actionName = "Manual";
    }

    if (!actionCounts[actionName]) {
      actionCounts[actionName] = { count: 0, hasFailed: false };
    }

    actionCounts[actionName]!.count += 1;

    // Track if any action in this group has failed
    if (action.state === ActionState.Failed) {
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
    if (action.componentId === hoveredComponentId.value && action.state === ActionState.Failed) {
      highlightedIds.add(action.id);
    }
  }
  return highlightedIds;
});

// ================================================================================================
// PINNING, RESOURCE COUNT, ETC.
const pinnedComponent = computed(() => {
  const mode = gridMode.value;
  if (mode.mode !== "pinned") return undefined;
  return componentList.value.find((c) => c.id === mode.componentId);
});
const connectionsGetter = useConnections();
const pinnedComponentConnections = computed(() =>
  // This is critical. We only want to get the connections if we found the pinned component. The ID
  // could have been provided via URL and the component may not exist anymore.
  pinnedComponent.value ? connectionsGetter(pinnedComponent.value.id).value : undefined,
);
const pinnedComponentConnectionSets = computed(() => {
  const incoming = new Set(pinnedComponentConnections.value?.incoming.map((c) => c.componentId) ?? []);
  const outgoing = new Set(pinnedComponentConnections.value?.outgoing.map((c) => c.componentId) ?? []);
  return {
    incoming,
    outgoing,
  };
});

const resourceCount = computed(() => componentList.value.filter((c) => c.hasResource).length ?? 0);
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
  let components = _.reverse(_.sortBy(filteredComponents.value ?? [], (c) => c.id));

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
      "Incoming subscriptions": [],
      "Outgoing subscriptions": [],
      "No subscriptions": [],
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
        groups["Incoming subscriptions"] ??= [];
        groups["Incoming subscriptions"].push(component);
        hasConnection = true;
      }
      if (pinnedComponentConnectionSets.value.outgoing.has(component.id)) {
        groups["Outgoing subscriptions"] ??= [];
        groups["Outgoing subscriptions"].push(component);
        hasConnection = true;
      }

      // Component is neither incoming nor outgoing, so it's not subscribed
      if (!hasConnection) {
        groups["No subscriptions"] ??= [];
        groups["No subscriptions"].push(component);
      }
    }
  } else if (gridMode.value.mode === "groupBy") {
    if (gridMode.value.criteria === "diff") {
      groups = {
        "With Diffs": [],
        "No Diffs": [],
      };
      for (const component of components) {
        const title = component.diffStatus && component.diffStatus !== "None" ? "With Diffs" : "No Diffs";
        groups[title]?.push(component);
      }
    } else if (gridMode.value.criteria === "qualification") {
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
    } else if (gridMode.value.criteria === "upgrade") {
      groups = {
        Upgradeable: [],
        "Up to date": [],
      };
      for (const component of components) {
        const title = upgradeableComponentIds.value.has(component.id) ? "Upgradeable" : "Up to date";
        groups[title]?.push(component);
      }
    } else if (gridMode.value.criteria === "resource") {
      groups = {
        "Has Resource": [],
        "No Resource": [],
      };
      for (const component of components) {
        const title = component.hasResource ? "Has Resource" : "No Resource";
        groups[title]?.push(component);
      }
    } else if (gridMode.value.criteria === "incompatibleComponents") {
      groups = {
        "Incompatible Components": [],
        "Compatible Components": [],
      };
      for (const component of components) {
        const title = component.hasSocketConnections ? "Incompatible Components" : "Compatible Components";
        groups[title]?.push(component);
      }
    } else if (gridMode.value.criteria === "schemaName") {
      const unsortedGroups: Record<string, ComponentInList[]> = {};
      for (const component of components ?? []) {
        const schemaName = component.schemaName;
        (unsortedGroups[schemaName] ??= []).push(component);
      }

      groups = Object.fromEntries(Object.entries(unsortedGroups).sort(([a], [b]) => a.localeCompare(b)));
    }
  } else if (gridMode.value.mode === "defaultSubscriptions") {
    const defaultSubs = defaultSubscriptions.value;
    groups = calculateDefaultSubscriptionGroups(componentsById.value, defaultSubs);
  } else {
    groups[""] = components;
  }

  return groups;
});

const calculateDefaultSubscriptionGroups = (
  componentMap: Record<ComponentId, ComponentInList>,
  defaultSubs: DefaultSubscriptions,
): Record<string, ComponentInList[]> => {
  const groups: Record<string, ComponentInList[]> = {};

  // If there are no default subscriptions, return an empty groups object
  if (defaultSubs.defaultSubscriptions.size === 0) {
    return groups;
  }

  const defaultSubComponentIds = [];
  const defaultSubsInverted: Record<ComponentId, string[]> = {};
  for (const [key, sub] of defaultSubs.defaultSubscriptions.entries()) {
    defaultSubComponentIds.push(sub.componentId);
    (defaultSubsInverted[sub.componentId] ??= []).push(key);
  }
  for (const keys of Object.values(defaultSubsInverted)) {
    keys.sort((a, b) =>
      (defaultSubs.defaultSubscriptions.get(a)?.path ?? "").localeCompare(
        defaultSubs.defaultSubscriptions.get(b)?.path ?? "",
      ),
    );
  }
  defaultSubComponentIds.sort((a, b) => -a.localeCompare(b));

  for (const defaultSubComponentId of defaultSubComponentIds) {
    for (const keyString of defaultSubsInverted[defaultSubComponentId] ?? []) {
      const sub = defaultSubs.defaultSubscriptions.get(keyString);
      if (!sub) {
        continue;
      }

      const componentIds = defaultSubs.componentsForSubs.get(keyString);
      if (componentIds) {
        const componentIdsAsArray = Array.from(componentIds);
        componentIdsAsArray.sort((a, b) => -a.localeCompare(b));

        const sourceComponent = componentMap[sub.componentId];
        if (!sourceComponent) {
          continue;
        }

        const componentsForGroup: ComponentInList[] = [];
        for (const componentId of componentIdsAsArray) {
          const component = componentMap[componentId];
          if (component) {
            componentsForGroup.push(component);
          }
        }

        groups[keyString] = componentsForGroup;
      } else {
        groups[keyString] = [];
      }
    }
  }

  return groups;
};

const MIN_GRID_TILE_WIDTH = 250;
const GRID_TILE_GAP = 16; // this is being used for both the X and Y gap
const scrollRef = ref<HTMLDivElement>();

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

// The expected number of components in a row based on the width of the scroll area
const virtualizerLanes = computed(() => {
  // We need to force a recompute of this value when the screen is resized
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  windowWidthReactive.value;

  // We also need to force a recompute of this value if the number of tiles changes
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  sortedAndGroupedComponents.value;

  // Our grid is based on the minimum tile width... so how many tiles can we fit?
  let newLanes = 0;
  let availableSpace = scrollRef.value?.getBoundingClientRect().width ?? 0;
  if (scrollRef.value && scrollRef.value.scrollHeight > scrollRef.value.clientHeight) {
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
const hasMultipleSections = computed(() => _.keys(sortedAndGroupedComponents.value).length > 1);

const collapseTracker = ref<Record<string, boolean>>({});

// Helper function to check if all components in a section are selected
const areAllComponentsInSectionSelected = (components: ComponentInList[], startIndex: number): boolean => {
  if (components.length === 0) return false;
  for (let i = 0; i < components.length; i++) {
    if (!selectedComponentIndexes.has(startIndex + i)) {
      return false;
    }
  }
  return true;
};

const gridRows = computed(() => {
  const rows: ExploreGridRowData[] = [];
  let dataIndex = 0;

  for (const groupName in sortedAndGroupedComponents.value) {
    const components = sortedAndGroupedComponents.value[groupName];
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

    // Check if all components in this section are selected
    const allSelected = areAllComponentsInSectionSelected(components, dataIndex);

    if (hasMultipleSections.value && gridMode.value.mode !== "defaultSubscriptions") {
      rows.push({
        type: "header",
        title: groupName,
        count,
        collapsed,
        allSelected,
      });
    } else if (gridMode.value.mode === "defaultSubscriptions") {
      const defaultSub: DefaultSubscription = defaultSubscriptions.value.defaultSubscriptions.get(groupName) ?? {
        componentId: "",
        path: "unknown",
      };

      const {
        name: componentName,
        schemaName,
        schemaCategory,
      } = componentsById.value[defaultSub.componentId] ?? {
        name: "Unknown",
        schemaName: "Unknown schema",
        schemaCategory: "Unknown category",
      };

      rows.push({
        type: "defaultSubHeader",
        schemaName,
        schemaCategory,
        componentName,
        componentId: defaultSub.componentId,
        path: defaultSub.path,
        collapsed,
        subKey: groupName,
        count,
        allSelected,
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
            insideSection: hasMultipleSections.value || gridMode.value.mode === "defaultSubscriptions",
          });

          // We need to increase the current index by the length of the row for the next iteration.
          dataIndex += components.length;
        }
      } else {
        rows.push({
          type: "emptyRow",
          groupName,
          insideSection: hasMultipleSections.value || gridMode.value.mode === "defaultSubscriptions",
        });
      }
    }

    // Whether or not we collapse the group, we need the footer.
    if (hasMultipleSections.value || gridMode.value.mode === "defaultSubscriptions") {
      rows.push({
        type: "footer",
      });
    }
  }

  // Remove the last footer when dealing with "group by" functionality.
  if (hasMultipleSections.value) rows.pop();

  // Add filtered counter row if needed
  if (
    shouldShowFilteredCounter.value &&
    filteredComponents.value !== undefined &&
    componentList.value.length !== undefined
  ) {
    const hiddenCount = componentList.value.length - filteredComponents.value.length;
    if (hiddenCount > 0) {
      rows.push({
        type: "filteredCounterRow",
        hiddenCount,
      });
    }
  }

  return rows;
});

const allVisibleComponents = computed(() => {
  // this excludes components which are inside collapsed groups
  const components: ComponentInList[] = [];
  for (const row of gridRows.value) {
    if (row.type === "contentRow") {
      components.push(...row.components);
    } else if (row.type === "pinnedContentRow") {
      components.push(row.component);
    }
  }
  return components;
});

const focusedComponent = computed(() => allVisibleComponents.value[focusedComponentIdx.value ?? -1]);
const selectedComponentsMap = computed(() => {
  const selected: Record<number, ComponentInList> = {};

  selectedComponentIndexes.forEach((index) => {
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

const focusedComponentRef = ref<HTMLElement | undefined>();

// ================================================================================================
// EXPLORE CONTEXT
const exploreContext = computed<ExploreContext>(() => {
  return {
    showSkeleton,
    lanesCount: virtualizerLanes,
    viewId: selectedViewOrDefaultId,
    selectedComponentIndexes,
    focusedComponentIdx,
    upgradeableComponents: upgradeableComponentIds,
    allVisibleComponents,
    selectedComponentsMap,
    focusedComponent,
    componentsHaveActionsWithState,
    componentsPendingActionNames,
    hasMultipleSections,
    focusedComponentRef,
    gridMode,
  };
});

provide("EXPLORE_CONTEXT", exploreContext.value);

// ================================================================================================
// THE SEARCH BAR AND FILTERING
// searchString can be null because VormInput sets the value to null onBlur if it's an empty string
const searchString = ref<string | null>("");
const showSearchFooter = ref(false);
const filteredComponentsRaw = useComponentSearch(() => searchString.value ?? "", componentList);
const filteredComponents = computed(() => filteredComponentsRaw.value ?? []);
const componentsById = computed(
  () =>
    filteredComponents.value?.reduce((accum, comp) => {
      accum[comp.id] = comp;
      return accum;
    }, {} as Record<ComponentId, ComponentInList>) ?? {},
);

// Filtered components counter state
const isScrolledToBottom = ref(false);
const shouldShowFilteredCounter = computed(() => {
  const hasFilteredComponents =
    filteredComponents.value && componentList.value.length > filteredComponents.value.length;
  return hasFilteredComponents && (isScrolledToBottom.value || (searchString.value ?? "").trim() !== "");
});

const resetFilter = () => {
  searchString.value = "";
  searchRef.value?.focus();
};

// Scroll detection variables
let scrollTimeout: ReturnType<typeof setTimeout> | null = null;
let lastScrollTime = 0;

const mapSelectedIds = ref<string[]>([]);
const onMapSelectedComponentsChange = (components: Set<ComponentInList>) => {
  const ids = [...components.values()].map((c) => c.id);
  mapSelectedIds.value = ids;
};

// Update the query of the route (allowing for URL links) when the group by selection change.
watch(
  () => [searchString, mapSelectedIds],
  () => {
    const query: SelectionsInQueryString = {
      ...router.currentRoute.value?.query,
    };

    if (!searchString.value) {
      // if search string is empty, remove it from the URL
      delete query.searchQuery;
    } else {
      query.searchQuery = searchString.value;
    }

    if (mapSelectedIds.value.length === 0) {
      delete query.c;
    } else {
      query.c = mapSelectedIds.value.join(",");
    }

    storeFilterAndGroup(query);
    router.replace({
      query,
    });
  },
  { deep: true },
);

// this is so that when on the map view, if you have a component selected
// and start searching, we clear the selected component
// on grid view, this has no impact, because you can't focus on the search box if you've got component(s) selected
watch(searchString, (newValue, oldValue) => {
  if (oldValue === "" && newValue === null) {
    return;
  }
  if (mapRef.value && typeof mapRef.value.deselect === "function") {
    mapRef.value.deselect();
  }
  clearSelection();
});

// ================================================================================================
// FOCUSING, TABBING, ETC.

watch(selectedComponentIndexes, () => {
  const ids = [...selectedComponentIndexes];
  const selectedURI = ids.join("|");

  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };

  if (selectedURI) query.s = selectedURI;
  else delete query.s;

  // Don't remove the s param if we're clearing selection while entering pinned mode
  // The gridMode watcher will handle updating the query with the pinned param
  if (!selectedURI && gridMode.value.mode === "pinned") {
    // Preserve the pinned state when clearing selections
    return;
  }

  storeFilterAndGroup(query);
  router.push({
    query,
  });
});

const focusedComponentIsPinned = computed(() => {
  if (gridMode.value.mode !== "pinned") return false;
  if (!focusedComponent.value) return false;
  return focusedComponent.value.id === gridMode.value.componentId;
});

const nextComponent = (wrap = false) => {
  if (!showGrid.value) return;

  if (focusedComponentIdx.value === undefined) {
    setFocusedComponentIdx(-1);
    return;
  }

  focusedComponentIdx.value += 1;

  if (focusedComponentIdx.value > allVisibleComponents.value.length - 1) {
    if (wrap) {
      setFocusedComponentIdx(-1);
    } else {
      setFocusedComponentIdx(allVisibleComponents.value.length - 1);
    }
  }
};
const previousComponent = (wrap = false) => {
  if (!showGrid.value) return;

  if (focusedComponentIdx.value === undefined) {
    setFocusedComponentIdx(-1);
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

  setFocusedComponentIdx(desiredIdx);
};

watch([focusedComponentIdx], () => {
  // console.log(focusedComponentIdx.value);
  if (focusedComponentIdx.value === -1) {
    searchRef.value?.focus();
  }
});

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

  const notRestorable = selectionComponentsForAction.value.find((component) => !component.toDelete);
  return notRestorable === undefined;
});

const fixContextMenu = async () => {
  if (bulkEditing.value) return;

  // We wait for three ticks to prevent the menu from opening
  // before the DOM scrolls to put the ExploreGridTile on screen
  // This eliminates menu flashing caused by the interaction between
  // fixing the menu's position and the automatic scroll behavior
  for (let i = 0; i < 3; i++) {
    await nextTick();
  }

  // If we focus on the pinned component, hide the context menu
  if (focusedComponentIsPinned.value) {
    componentContextMenuRef.value?.close();
  } else if (
    exploreContext.value.focusedComponentRef.value &&
    selectionComponentsForAction.value &&
    !gridScrolling.value
  ) {
    componentContextMenuRef.value?.open(
      exploreContext.value.focusedComponentRef.value,
      selectionComponentsForAction.value,
    );
  }
};
const clearSelection = () => {
  selectedComponentIndexes.clear();
  bulkEditing.value = false;
  if (focusedComponentIdx.value === -1) {
    setFocusedComponentIdx(-1);
    componentContextMenuRef.value?.close();
  } else {
    unfocus();
  }
};

// This function is fired when a ExploreGridTile checkbox is clicked
// Or for complex selection logic resulting from holding shift/cmd/ctrl
const selectComponent = (componentIdx: number, event?: MouseEvent) => {
  if (event?.shiftKey) {
    // Shift key behavior takes priority first
    if (focusedComponentIdx.value !== undefined && selectedComponentIndexes.size === 0) {
      // If a component is focused but nothing is selected, add it to the selection
      selectedComponentIndexes.add(focusedComponentIdx.value);
    } else if (event.button === 0 && selectedComponentIndexes.has(componentIdx)) {
      // If shift left click and clicked component is already selected, deselect it
      deselectComponent(componentIdx);
      return; // do not continue in this case!
    }

    if (selectedComponentIndexes.size > 0) {
      // Range selection with shift key
      const selectedIndexes = Array.from(selectedComponentIndexes);
      const lastSelectedIdx = Math.max(...selectedIndexes);
      const start = Math.min(lastSelectedIdx, componentIdx);
      const end = Math.max(lastSelectedIdx, componentIdx);

      // Select all components in the range
      for (let i = start; i <= end; i++) {
        selectedComponentIndexes.add(i);
      }
    } else {
      // Add component to selected list (shift click with no selection)
      selectedComponentIndexes.add(componentIdx);
    }
  } else if (event?.ctrlKey || event?.metaKey) {
    // Pick - select or deselect invidual components
    if (selectedComponentIndexes.has(componentIdx) && event.button === 0) {
      // Remove component from selected list
      deselectComponent(componentIdx);
      return; // do not continue in this case!
    } else {
      // Add component to selected list
      selectedComponentIndexes.add(componentIdx);
    }
  } else {
    // Add component to selected list (checkbox click)
    selectedComponentIndexes.add(componentIdx);
  }

  // finally pop context menu if right click or menu is already open
  if (event?.button === 2 || focusedComponentIdx.value !== undefined) {
    setFocusedComponentIdx(componentIdx);
  }
};

// This function just deselects a component, no other logic inside
const deselectComponent = (componentIdx: number | string) => {
  // PSA: componentIdx coming through emits is typed as number, but at execution is a string
  if (typeof componentIdx === "string") componentIdx = parseInt(componentIdx);
  selectedComponentIndexes.delete(componentIdx);

  // If we're deselecting the currently focused component and there are still other selections,
  // we need to transfer focus to another selected component
  if (focusedComponentIdx.value === componentIdx && selectedComponentIndexes.size > 0) {
    // Find the highest index from remaining selections to focus on
    const remainingIndexes = Array.from(selectedComponentIndexes);
    const newFocusedIdx = Math.max(...remainingIndexes);
    setFocusedComponentIdx(newFocusedIdx);
    // The fixContextMenu will be called by the watcher on focusedComponent
  }

  // Clear selection entirely if no more selections remain
  if (selectedComponentIndexes.size === 0) {
    clearSelection();
  }

  // If we still have selected components, fix the menu's selection accordingly
  if (selectionComponentsForAction.value) {
    componentContextMenuRef.value?.setSelectedComponents(selectionComponentsForAction.value);
  }
};

const isComponentSelected = (componentIdx: number) => selectedComponentIndexes.has(componentIdx);

const selectAllInSection = (sectionKey: string) => {
  // Find all components in the section by looking at gridRows
  const componentsToSelect: number[] = [];

  for (const row of gridRows.value) {
    if (row.type === "contentRow") {
      // Check if this contentRow belongs to the section we're looking for
      // We need to track which section we're in by looking at headers
      const currentSectionKey = findSectionKeyForDataIndex(row.chunkInitialId);

      if (currentSectionKey === sectionKey) {
        // Add all component indexes in this row
        for (let i = 0; i < row.components.length; i++) {
          componentsToSelect.push(row.chunkInitialId + i);
        }
      }
    }
  }

  // Select all components in the section without focusing
  // This allows the context menu to only appear on right-click
  componentsToSelect.forEach((idx) => selectComponent(idx));
};

const deselectAllInSection = (sectionKey: string) => {
  // Find all components in the section by looking at gridRows
  const componentsToDeselect: number[] = [];

  for (const row of gridRows.value) {
    if (row.type === "contentRow") {
      // Check if this contentRow belongs to the section we're looking for
      const currentSectionKey = findSectionKeyForDataIndex(row.chunkInitialId);

      if (currentSectionKey === sectionKey) {
        // Add all component indexes in this row
        for (let i = 0; i < row.components.length; i++) {
          componentsToDeselect.push(row.chunkInitialId + i);
        }
      }
    }
  }

  // Deselect all components in the section
  componentsToDeselect.forEach((idx) => deselectComponent(idx));
};

const findSectionKeyForDataIndex = (dataIndex: number): string | undefined => {
  let currentSection: string | undefined;

  for (const row of gridRows.value) {
    if (row.type === "header") {
      currentSection = row.title;
    } else if (row.type === "defaultSubHeader") {
      currentSection = row.subKey;
    } else if (row.type === "contentRow") {
      if (row.chunkInitialId === dataIndex) {
        return currentSection;
      }
    }
  }

  return currentSection;
};

// ================================================================================================
// CLICKING AND NAVIGATION
const componentClicked = (e: MouseEvent, componentId: ComponentId, componentIdx: number) => {
  e.preventDefault();
  if (e.shiftKey || e.metaKey || e.ctrlKey) {
    // Complex selection logic here!
    selectComponent(componentIdx, e);
    return;
  }

  // Basic selection logic here
  if (e.button === 0) {
    // Left-click: just navigate, don't affect selections
    componentNavigate(componentId);
  } else if (e.button === 2) {
    // Right-click: if component isn't already selected, select it first
    if (!isComponentSelected(componentIdx)) {
      clearSelection();
      selectedComponentIndexes.add(componentIdx);
    }
    setFocusedComponentIdx(componentIdx);
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
  fixContextMenu();
});

// ================================================================================================
// KEYBOARD NAVIGATION
const searchRef = ref<InstanceType<typeof VormInput>>();
const mountEmitters = () => {
  removeEmitters();
  for (const [key, func] of Object.entries(shortcuts)) {
    keyEmitter.on(key, func);
  }
  windowResizeEmitter.on("resize", onResize);
};
const removeEmitters = () => {
  for (const [key, func] of Object.entries(shortcuts)) {
    keyEmitter.off(key, func);
  }
  windowResizeEmitter.off("resize", onResize);
};

const onArrow = () => {
  // If the context menu is open and has no focus, focus its first item!
  // All other arrow key controls are managed by the menu itself.
  componentContextMenuRef.value?.focusFirstItem(true);
};

// eslint-disable-next-line @typescript-eslint/no-duplicate-type-constituents
const onBackspace = (e: KeyDetails["Backspace"] | KeyDetails["Delete"]) => {
  e.preventDefault();

  if (showGrid.value) {
    if (!selectionComponentsForAction.value) return;
    componentContextMenuRef.value?.componentsStartDelete(selectionComponentsForAction.value);
  } else {
    mapRef.value?.onBackspace(e);
  }
};

const onTab = (e: KeyDetails["Tab"], blurSearch = false) => {
  e.preventDefault();
  if (!showGrid.value) return; // no tab behavior on the map yet
  if (isThereAModalOpen.value) return; // no tab behavior when a modal is open

  selectedComponentIndexes.clear();
  bulkEditing.value = false;
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

const onEscape = () => {
  if (isThereAModalOpen.value || bulkEditing.value) return;

  if (showGrid.value) {
    searchRef.value?.blur();
    clearSelection();
  } else {
    mapRef.value?.onEscape();
  }
};

const openShortcutModal = () => {
  shortcutModalRef.value?.open();
};

const shortcuts: { [Key in string]: (e: KeyDetails[Key]) => void } = {
  a: (e) => {
    e.preventDefault();
    if (e.metaKey || e.ctrlKey) {
      const components = allVisibleComponents.value;
      let lastIndex = -1;
      [...components.keys()].forEach((index) => {
        const component = componentList.value[index];
        if (component) {
          selectComponent(index);
          lastIndex = index;
        }
      });
      // Only set focus once after selecting all components to avoid race conditions
      if (lastIndex !== -1) {
        setFocusedComponentIdx(lastIndex);
      }
    }
  },
  b: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();
    if (showGrid.value && selectionComponentsForAction.value && selectionComponentsForAction.value.length > 1) {
      startBulkEdit();
    }
  },
  c: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();
    emit("openChangesetModal");
  },
  d: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();

    if (showGrid.value) {
      if (!selectionComponentsForActionIds.value) return;
      componentContextMenuRef.value?.duplicateComponentStart(selectionComponentsForActionIds.value);
    } else {
      mapRef.value?.onD(e);
    }
  },
  e: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();
    if (showGrid.value) {
      if (!selectionComponentsForAction.value) return;

      componentContextMenuRef.value?.componentsStartErase(selectionComponentsForAction.value);
    } else {
      mapRef.value?.onE(e);
    }
  },
  f: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    if (showGrid.value) {
      if (!selectionComponentsForActionIds.value) return;
      if (allSelectedComponentsAreRestorable.value) {
        componentContextMenuRef.value?.componentsRestore(selectionComponentsForActionIds.value);
      }
    } else {
      mapRef.value?.onR(e);
    }
  },
  // g: undefined,
  // h: undefined,
  // i: used for import in ComponentDetails
  // j: undefined,
  k: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();

    // Deselect the current selection based on which screen you are on
    if (showGrid.value) {
      clearSelection();
    } else {
      mapRef.value?.deselect();
    }

    // same behavior on the grid and map!
    searchRef.value?.focus();
  },
  // l: undefined,
  m: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();
    if (showGrid.value) {
      // Do nothing in grid mode
      return;
    }
    mapRef.value?.onM(e);
  },
  n: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();

    // same behavior on the grid and map!
    openAddComponentModal();
  },
  // o: undefined,
  p: (e) => {
    // You can only pin one component at a time!
    if (selectedComponentIndexes.size > 1) return;
    else if (e.metaKey || e.ctrlKey) return;

    e.preventDefault();
    if (showGrid.value) {
      if (!focusedComponent.value || selectedComponents.value.length > 1) return;

      // We do not need the context menu to pin and unpin.
      if (focusedComponentIsPinned.value) {
        gridMode.value = { mode: "default", label: "" };
      } else {
        gridMode.value = {
          mode: "pinned",
          label: "",
          componentId: focusedComponent.value.id,
        };
      }
    } else {
      mapRef.value?.onP(e);
    }
  },
  // q: undefined,
  r: (e) => {
    if (e.metaKey || e.ctrlKey) {
      // This is the chrome hotkey combo for refreshing the page! Let it happen!
      return;
    } else if (ctx.onHead.value) {
      // Can't open the review screen on Head
      return;
    }

    e.preventDefault();
    router.push({
      name: "new-hotness-review",
    });
  },
  // s: undefined,
  u: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();

    if (showGrid.value) {
      if (!selectionComponentsForActionIds.value) return;

      if (allSelectedComponentsAreUpgradeable.value) {
        componentContextMenuRef.value?.componentsUpgrade(selectionComponentsForActionIds.value);
      }
    } else {
      mapRef.value?.onU(e);
    }
  },
  // v: undefined, - USED FOR VIEWS BUTTON IN ComponentDetails
  // w: undefined,
  // x: undefined,
  // y: undefined,
  // z: undefined,

  ArrowRight: onArrow,
  ArrowLeft: onArrow,
  // Up and Down arrows are used by the ComponentContextMenu
  Enter: (e) => {
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
  },
  " ": () => {
    // For non-pinned components, the ComponentContextMenu handles the spacebar press
    // But for the pinned component, we need to handle it here
    if (focusedComponentIdx.value === 0 && focusedComponentIsPinned.value) {
      navigateToFocusedComponent();
    }
  },
  Tab: onTab,
  Escape: onEscape,
  Backspace: onBackspace,
  Delete: onBackspace,
  "/": openShortcutModal,
  "?": openShortcutModal,
};

// ================================================================================================
// SCROLLING AND CLICKING
const gridScrolling = ref(false);
const onScroll = (event: Event) => {
  componentContextMenuRef.value?.close();
  gridScrolling.value = true;

  const target = event.target as HTMLElement;
  if (!target) return;

  const { scrollTop, scrollHeight, clientHeight } = target;
  const now = Date.now();
  lastScrollTime = now;

  if (scrollTimeout) {
    clearTimeout(scrollTimeout);
  }

  scrollTimeout = setTimeout(() => {
    if (now !== lastScrollTime) return;

    const currentlyAtBottom = isScrolledToBottom.value;
    const nearBottom = scrollTop + clientHeight >= scrollHeight - 200;

    let shouldShow = nearBottom;
    if (currentlyAtBottom && !nearBottom) {
      shouldShow = scrollTop + clientHeight >= scrollHeight - 300;
    }

    if (isScrolledToBottom.value !== shouldShow) {
      isScrolledToBottom.value = shouldShow;
    }
  }, 250);
};

const onScrollEnd = async () => {
  gridScrolling.value = false;

  if (focusedComponentIsPinned.value) {
    componentContextMenuRef.value?.close();
    return;
  }

  // We need to fix the context menu after scrolling!
  // If the element is scrolled into view, show the menu
  // If the element is scrolled offscreen, unfocus and reset selected component index
  const el = exploreContext.value.focusedComponentRef.value;
  if (el && elementIsScrolledIntoView(el)) {
    fixContextMenu();
  } else {
    unfocus();
    // Don't automatically set focus for selections - context menu should only show on right-click
  }
};
const onResize = () => {
  if (bulkEditing.value) return;
  clearSelection();
};

// general click handler for the whole page
// any click which doesn't do this behavior should have .stop on it!
const onClick = (e: MouseDetails["click"]) => {
  if (bulkEditing.value) return;

  if (showGrid.value) {
    const inside = componentContextMenuRef.value?.contextMenuRef?.elementIsInsideMenu;
    if (inside && e.target instanceof Node && inside(e.target)) {
      return;
    }
    clearSelection();
  }
};

// ================================================================================================
// MOUNTING AND URL QUERY HANDLING
const setSelectionsFromQuery = async () => {
  const query: SelectionsInQueryString = router.currentRoute.value?.query;

  // if we get retainSessionState, get query back from the session storage
  if (query.retainSessionState) {
    const query = retrieveFilterAndGroup();
    delete query.retainSessionState;
    router.replace({ query });

    return;
  }

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

  // Only set gridMode if there's an explicit groupBy value
  // Don't reset to default here - let pinned/defaultSubscriptions checks below handle it
  if (query.groupBy) {
    switch (query.groupBy) {
      case "diffstatus":
        gridMode.value = {
          mode: "groupBy",
          criteria: "diff",
          label: "Diff Status",
        };
        break;
      case "qualificationstatus":
        gridMode.value = {
          mode: "groupBy",
          criteria: "qualification",
          label: "Qualification Status",
        };
        break;
      case "upgradeable":
        gridMode.value = {
          mode: "groupBy",
          criteria: "upgrade",
          label: "Upgradeable",
        };
        break;
      case "schemaname":
        gridMode.value = {
          mode: "groupBy",
          criteria: "schemaName",
          label: "Schema Name",
        };
        break;
      case "resource":
        gridMode.value = {
          mode: "groupBy",
          criteria: "resource",
          label: "Resource",
        };
        break;
      case "incompatibleComponents":
        gridMode.value = {
          mode: "groupBy",
          criteria: "incompatibleComponents",
          label: "Incompatible Components",
        };
        break;
      default:
        // Unknown groupBy value, don't set gridMode
        break;
    }
  } else if (!query.pinned && query.defaultSubscriptions !== "1") {
    // Only reset to default if there's no groupBy, pinned, or defaultSubscriptions
    gridMode.value = { mode: "default", label: "" };
  }

  if (query.pinned !== undefined) {
    gridMode.value = { mode: "pinned", label: "", componentId: query.pinned };
  }

  if (query.defaultSubscriptions === "1") {
    gridMode.value = { mode: "defaultSubscriptions", label: "" };
  }

  if (query.viewId !== undefined) {
    selectedViewId.value = query.viewId;
  }

  if (query.s) {
    const indexes = new Set(query.s.split("|").map((idx) => parseInt(idx)));
    selectedComponentIndexes.clear();
    indexes.forEach((idx) => {
      selectedComponentIndexes.add(idx);
    });
    const idx = [...indexes].pop();
    // NOTE: Only set focus when actually loading from URL on mount, not during checkbox interactions
    // Don't set focus if we already have selections (indicating this is a checkbox interaction, not URL load)
    if (idx !== undefined && focusedComponentIdx.value === undefined && selectedComponentIndexes.size === 0) {
      // when we're on mount, we need to wait until the next tick for the ref
      // yes, there are 2 on purpose
      await nextTick();
      selectComponent(idx);
      setFocusedComponentIdx(idx); // Set focus for URL-based selection
    }
  } else delete query.s;

  if (query.b && query.b === "1" && selectedComponentIndexes.size > 0) bulkEditing.value = true;
  else {
    bulkEditing.value = false;
    delete query.b;
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
  if (scrollTimeout) {
    clearTimeout(scrollTimeout);
  }
});

// without this watch the `retainSessionState` functionality doesn't fire
// perhaps that can live in `beforeEnter` for the route?
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
// SORT BY STUFF
export type SortByUrlQuery = "failingactions" | "runningactions";

const sortByFromString = (s: string): SortByCriteria => {
  const key = (_.keys(SortByCriteria) as (keyof typeof SortByCriteria)[]).find(
    // eslint-disable-next-line @typescript-eslint/no-unsafe-enum-comparison
    (k) => SortByCriteria[k] === s,
  );

  if (!key) return SortByCriteria.LatestToOldest;
  else return SortByCriteria[key];
};

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

  storeFilterAndGroup(query);
  router.push({
    query,
  });
});

const onMenuFinishAction = () => {
  clearSelection();
};

// ================================================================================================
// EMITS AND THE REST
const emit = defineEmits<{
  (e: "openChangesetModal"): void;
}>();
</script>

<style lang="css" scoped>
section.grid.explore {
  grid-template-columns: minmax(0, 75%) minmax(0, 25%);
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
