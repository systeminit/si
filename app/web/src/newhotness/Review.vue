<template>
  <div class="w-full h-full flex flex-col">
    <div
      :class="
        clsx(
          'header flex-none flex flex-row items-center gap-xs px-sm py-xs border-b',
          themeClasses(
            'bg-white border-neutral-300',
            'bg-neutral-800 border-neutral-600',
          ),
        )
      "
    >
      <NewButton
        tooltip="Close (Esc)"
        tooltipPlacement="top"
        icon="x"
        tone="empty"
        :class="
          clsx(
            'active:bg-white active:text-black',
            themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-600'),
          )
        "
        @click="exitReview"
      />
      <div class="flex-1 text-sm font-medium">Review Changes</div>
      <div v-if="!ctx.onHead.value" class="flex gap-xs items-center">
        <div
          v-if="filteredComponentList?.length > 0"
          :class="
            clsx(
              'text-sm px-xs py-2xs rounded',
              themeClasses(
                'text-neutral-600 bg-neutral-100',
                'text-neutral-400 bg-neutral-700',
              ),
            )
          "
        >
          {{
            selectedComponentId
              ? `${currentComponentIndex + 1} / ${filteredComponentList.length}`
              : filteredComponentList.length
          }}
        </div>
        <NewButton
          label="Previous"
          :disabled="!canGoBack"
          @click.stop.prevent="goToPreviousComponent"
        >
          <template #icon>
            <div class="border border-neutral-400 rounded p-3xs mr-2xs">
              <Icon name="arrow--left" size="xs" />
            </div>
          </template>
        </NewButton>
        <NewButton
          label="Next"
          tone="action"
          :disabled="!canGoForward"
          @click.stop.prevent="goToNextComponent"
        >
          <template #iconRight>
            <div class="border border-action-200 rounded p-3xs ml-2xs">
              <Icon name="arrow--right" size="xs" />
            </div>
          </template>
        </NewButton>
      </div>
    </div>
    <section
      v-if="ctx.onHead.value"
      class="p-lg flex flex-col items-center gap-sm"
    >
      <EmptyState
        icon="x"
        text="You are on HEAD"
        secondaryText="There are no changes to review"
      />
      <NewButton label="Exit" icon="chevron--left" @click="exitReview" />
    </section>
    <section v-else class="grid review w-full min-h-0 grow flex-1 p-xs">
      <div
        :class="
          clsx(
            'left',
            'flex flex-col gap-xs m-xs p-xs border',
            themeClasses(
              'border-neutral-400 bg-white',
              'border-neutral-600 bg-neutral-800',
            ),
          )
        "
      >
        <div class="text-sm flex-none">Components Changed</div>
        <SiSearch
          ref="searchRef"
          v-model="searchString"
          class="flex-none"
          variant="new"
          placeholder="Find a component"
          :borderBottom="false"
          @focus="() => (selectedComponentId = undefined)"
          @keydown.tab="onSearchTab"
          @keydown.up="() => searchControl(true)"
          @keydown.down="() => searchControl(false)"
        />
        <div
          ref="componentListRef"
          class="flex flex-col gap-xs flex-grow scrollable"
        >
          <EmptyState
            v-if="componentList.length === 0"
            icon="diff"
            text="No components changed"
            class="p-sm"
          />
          <EmptyState
            v-else-if="filteredComponentList?.length === 0"
            icon="diff"
            text="No changed components match your search"
            class="p-sm"
          />
          <ComponentListItem
            v-for="component in addedComponentList"
            :key="component.id"
            :component="component"
            status="Added"
            :selected="component.id === selectedComponentId"
            :data-component-id="component.id"
            @click="selectComponent(component.id)"
          />
          <ComponentListItem
            v-for="component in modifiedComponentList"
            :key="component.id"
            :component="component"
            status="Modified"
            :selected="component.id === selectedComponentId"
            :data-component-id="component.id"
            @click="selectComponent(component.id)"
          />
          <ComponentListItem
            v-for="component in removedComponentList"
            :key="component.id"
            :component="component"
            status="Removed"
            :selected="component.id === selectedComponentId"
            :data-component-id="component.id"
            @click="selectComponent(component.id)"
          />
        </div>
      </div>
      <div class="main flex flex-col gap-sm m-xs">
        <CollapsingFlexItem
          v-if="selectedComponentId && selectedComponent"
          disableCollapse
          headerTextSize="sm"
        >
          <template #header>
            <div
              class="group/title flex flex-row items-center gap-xs w-full cursor-pointer"
              @click="goToComponentDetails"
            >
              <TruncateWithTooltip
                :class="
                  clsx(
                    'py-2xs max-w-fit flex-1 group-hover/title:underline',
                    themeClasses(
                      'text-neutral-800 group-hover/title:text-action-500',
                      'text-neutral-100 group-hover/title:text-action-300',
                    ),
                  )
                "
              >
                {{ selectedComponent.schemaName }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'py-2xs max-w-fit flex-1 group-hover/title:underline',
                    themeClasses(
                      'text-neutral-600 group-hover/title:text-action-500',
                      'text-neutral-400 group-hover/title:text-action-300',
                    ),
                  )
                "
              >
                ({{ selectedComponent.name }})
              </TruncateWithTooltip>
            </div>
          </template>

          <div class="flex flex-col gap-sm px-sm py-sm min-h-full">
            <div
              v-if="selectedComponent.diffStatus === 'Removed'"
              :class="
                clsx(
                  'flex flex-row items-center gap-xs p-xs text-sm',
                  themeClasses(
                    'text-neutral-800 bg-neutral-300',
                    'text-neutral-100 bg-neutral-600',
                  ),
                )
              "
            >
              <template v-if="selectedComponent.toDelete">
                <div class="mr-auto">
                  This component will be removed from HEAD once the current
                  change set is applied.
                </div>
                <NewButton
                  v-if="
                    selectedComponent.toDelete &&
                    restoreComponentStatus !== 'succeeded'
                  "
                  label="Restore"
                  :loading="restoreComponentStatus === 'inProgress'"
                  loadingIcon="loader"
                  loadingText="Restoring..."
                  @click="restoreComponent"
                />
              </template>
              <div v-else>
                This component will be removed from HEAD without queueing a
                delete action once the current change set is applied. This
                cannot be undone within this change set.
              </div>
            </div>

            <!-- Show /si/name-->
            <ReviewAttributeItem
              v-if="
                selectedComponent.attributeDiffTree?.children?.si?.children
                  ?.name
              "
              :selectedComponentId="selectedComponentId"
              name="name"
              :item="
                selectedComponent.attributeDiffTree.children.si.children.name
              "
              :disableRevert="disableRevert"
            />
            <!-- Show children of /si/domain -->
            <template
              v-if="
                selectedComponent.attributeDiffTree?.children?.domain?.children
              "
            >
              <ReviewAttributeItem
                v-for="(item, name) in selectedComponent.attributeDiffTree
                  .children.domain.children"
                :key="name"
                :selectedComponentId="selectedComponentId"
                :name="name"
                :item="item"
                :disableRevert="disableRevert"
              />
            </template>
            <!-- Show children of /si/secrets -->
            <template
              v-if="
                selectedComponent.attributeDiffTree?.children?.secrets?.children
              "
            >
              <ReviewAttributeItem
                v-for="(item, name) in selectedComponent.attributeDiffTree
                  .children.secrets.children"
                :key="name"
                :selectedComponentId="selectedComponentId"
                :name="name"
                :item="item"
                :disableRevert="disableRevert"
              />
            </template>

            <div
              v-if="noAVDiffs"
              :class="
                clsx(
                  'w-full grow flex flex-row items-center justify-center border',
                  themeClasses('border-neutral-400', 'border-neutral-600'),
                )
              "
            >
              <EmptyState
                icon="diff"
                text="No Attribute Values changed"
                secondaryText="There are no attribute value changes to display for this component"
              />
            </div>
          </div>
        </CollapsingFlexItem>
        <div
          v-else
          :class="
            clsx(
              'border grow flex flex-col items-center justify-center',
              themeClasses(
                'border-neutral-400 bg-white',
                'border-neutral-600 bg-neutral-800',
              ),
            )
          "
        >
          <EmptyState
            icon="component"
            text="No component selected"
            secondaryText="Select a component to see information about it"
          />
        </div>
        <CollapsingFlexItem
          ref="actionsRef"
          headerTextSize="sm"
          maxHeightContent
          :expandable="false"
        >
          <template #header> Actions </template>
          <!--
        For anything related to actions, check if we have both the "selectedComponentId" and the
        "details" to make sure the data comes in atomically and with reactivity.
        -->
          <template v-if="selectedComponent" #headerIcons>
            <ActionPills
              :actionCounts="actionCounts"
              mode="row"
              showNoPendingActions
            />
          </template>
          <template v-if="selectedComponent">
            <ActionsPanel
              ref="actionsPanelRef"
              :component="selectedComponent"
            />
          </template>
          <EmptyState
            v-else
            class="p-lg"
            icon="component"
            text="No component selected"
            secondaryText="Select a component to see and configure actions"
          />
        </CollapsingFlexItem>
      </div>
      <div class="right flex flex-col p-xs">
        <CollapsingFlexItem open headerTextSize="sm">
          <template #header>Component History</template>
          <template v-if="selectedComponentId">
            <ComponentHistory :componentId="selectedComponentId" />
          </template>
          <EmptyState
            v-else
            class="p-lg"
            icon="component"
            text="No component selected"
            secondaryText="Select a component to see its history"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem open headerTextSize="sm">
          <template #header>Diff</template>
          <CodeViewer
            v-if="selectedComponent"
            :title="`${selectedComponent.name}: ${selectedComponent.schemaName}`"
            :code="
              selectedComponent.componentDiff?.resourceDiff?.diff ||
              selectedComponent.componentDiff?.resourceDiff?.current
            "
            codeLanguage="diff"
            copyTooltip="Copy diff to clipboard"
          />
          <EmptyState
            v-else
            class="p-lg"
            icon="component"
            text="No component selected"
            secondaryText="Select a component to see the diff for it"
          />
        </CollapsingFlexItem>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { useQueries, useQuery, useQueryClient } from "@tanstack/vue-query";
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import clsx from "clsx";
import {
  Icon,
  SiSearch,
  themeClasses,
  TruncateWithTooltip,
  NewButton,
} from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import * as _ from "lodash-es";
import { sleep } from "@si/ts-lib/src/async-sleep";
import {
  bifrost,
  bifrostList,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import {
  ActionDiffList,
  ActionDiffView,
  AttributeDiff,
  AttributeSourceAndValue,
  ComponentDiff,
  ComponentInList,
  EntityKind,
  ErasedComponents,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { ActionState } from "@/api/sdf/dal/action";
import ComponentListItem from "./ComponentListItem.vue";
import ActionsPanel from "./ActionsPanel.vue";
import ActionPills from "./ActionPills.vue";
import { useContext } from "./logic_composables/context";
import EmptyState from "./EmptyState.vue";
import ComponentHistory from "./ComponentHistory.vue";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import ReviewAttributeItem from "./ReviewAttributeItem.vue";
import { KeyDetails, keyEmitter } from "./logic_composables/emitters";
import { useComponentSearch } from "./logic_composables/search";
import { useComponentActions } from "./logic_composables/component_actions";
import { useComponentDeletion } from "./composables/useComponentDeletion";

const ctx = useContext();

const router = useRouter();
const route = useRoute();
const queryClient = useQueryClient();

// const changeSetName = computed(() => ctx.changeSet.value?.name);
const selectedComponentId = ref<ComponentId>();

// Initialize selected component from URL query parameter
const initializeFromUrl = () => {
  const componentIdFromUrl = route.query.component as ComponentId;
  if (componentIdFromUrl) {
    selectedComponentId.value = componentIdFromUrl;
  } else {
    // TODO(Wendy) - remove this part to not have the first component selected by default
    nextTick(() => {
      if (componentList.value[0]?.id) {
        selectedComponentId.value = componentList.value[0].id;
      }
    });
  }
};

const key = useMakeKey();
const args = useMakeArgs();

// All components on this change set
const changeSetComponentListQuery = useQuery({
  queryKey: key(EntityKind.ComponentList),
  enabled: ctx.queriesEnabled,
  queryFn: async () =>
    await bifrostList<ComponentInList[]>(args(EntityKind.ComponentList)),
});
const changeSetComponentList = computed(
  () => changeSetComponentListQuery.data.value ?? [],
);

/** Queries for complete attribute-by-attribute diff of every component */
const componentDiffQueries = useQueries({
  queries: computed(() =>
    changeSetComponentList.value.map((component) => ({
      queryKey: key(EntityKind.ComponentDiff, component.id),
      queryFn: async () =>
        await bifrost<ComponentDiff>(
          args(EntityKind.ComponentDiff, component.id),
        ),
    })),
  ),
});

const erasedComponents = useQuery({
  queryKey: key(EntityKind.ErasedComponents),
  enabled: ctx.queriesEnabled,
  queryFn: async () =>
    await bifrost<ErasedComponents>(args(EntityKind.ErasedComponents)),
});

/** Query to get actions that have changed relative to HEAD. */
const actionDiffListQuery = useQuery({
  queryKey: key(EntityKind.ActionDiffList),
  enabled: ctx.queriesEnabled,
  queryFn: async () =>
    await bifrost<ActionDiffList>(args(EntityKind.ActionDiffList)),
});

/**
 * The complete component list without diff information added yet
 * We need this computed so that we can get info about the components
 * inside of the computed "componentList"
 */
const rawComponentList = computed(() => {
  const result = changeSetComponentList.value.map((component) => {
    return {
      ...component,
    };
  });

  return result;
});

/**
 * Complete component list, including:
 * - Added and modified components from the changeset
 * - Removed components from HEAD
 * - `diff` set to the ComponentDiff MV for the component (if found)
 * - `diffStatus` reflecting the status from the ComponentDiff MV
 *
 * TODO we will want to make the diffStatus correct in the ComponentList MV in the first place,
 * so we don't have to "fix" it here
 */
const componentList = computed(() => {
  const componentActionDiffs: { [id in ComponentId]?: ActionDiffView[] } = {};
  for (const actionDiff of Object.values(
    actionDiffListQuery.data.value?.actionDiffs ?? [],
  )) {
    if (actionDiff.diffStatus !== "None") {
      componentActionDiffs[actionDiff.componentId] ??= [];
      componentActionDiffs[actionDiff.componentId]?.push(actionDiff);
    }
  }

  // Add component and action diffs to each component in the change set, and include "removed"
  const componentDiffs: { [id in ComponentId]?: ComponentDiff } =
    Object.fromEntries(
      componentDiffQueries.value.map(
        (query) => [query.data?.id, query.data] as const,
      ),
    );
  const mapped = rawComponentList.value
    .map((component) => {
      const componentDiff = componentDiffs[component.id];
      const actionDiffs = componentActionDiffs[component.id];
      const attributeDiffTree = toAttributeDiffTree(componentDiff);

      // Figure out diffStatus
      let diffStatus = componentDiff?.diffStatus;
      // If the diffStatus *was* Modified, but none of the diffs were worth showing, then we
      // don't want to show it. (If it's Added or Removed, or has other things that are worth
      // showing, we will bring the status back in the next few.)
      if (
        diffStatus === "Modified" &&
        !attributeDiffTree.diff &&
        !attributeDiffTree.children
      ) {
        diffStatus = "None";
      }

      // If we don't have a ComponentDiff diffStatus, fall back to the ComponentInList's diffStatus
      diffStatus ??= component.diffStatus;

      // If there are diffs, we are Modified
      if (diffStatus === "None" && actionDiffs && actionDiffs?.length > 0) {
        diffStatus = "Modified";
      }

      // If it's toDelete, put it in the Removed category
      if (component.toDelete) {
        diffStatus = "Removed";
      }

      return {
        ...component,
        diffStatus,
        componentDiff,
        attributeDiffTree,
        actionDiffs,
      };
    })
    .filter((component) => component.diffStatus !== "None");

  // Add erased components
  for (const { diff, component } of Object.values(
    erasedComponents.data.value?.erased ?? {},
  )) {
    const actionDiffs = componentActionDiffs[component.id];
    const attributeDiffTree = toAttributeDiffTree(diff);
    mapped.push({
      ...component,
      diffStatus: "Removed",
      componentDiff: diff,
      attributeDiffTree,
      actionDiffs,
    });
  }
  return mapped;
});

/**
 * A tree version of AttributeDiff:
 *
 *     {
 *       children: {
 *         domain: {
 *           children: {
 *             SubnetIds: {
 *               children: {
 *                 0: { diff: ... }
 *               }
 *               // parents may or may not have a diff! Especially if they have a *source* difference
 *               diff: {...},
 *             },
 *             extra: {
 *               children: {
 *                 Region: { diff: {} }
 *               }
 *             }
 *           }
 *         }
 *       }
 *     }
 */
export interface AttributeDiffTree {
  path: AttributePath;
  diff?: AttributeDiff;
  children?: Record<string, AttributeDiffTree>;
}

// TEMPORARY: post-process the ComponentDiff MV:
// - add nesting when there are both child and parent values in the diff
// - correct for a bug where subscriptions weren't showing right for some reason
function toAttributeDiffTree(componentDiff?: ComponentDiff): AttributeDiffTree {
  const tree: AttributeDiffTree = { path: "" as AttributePath };

  const attributeDiffs = componentDiff?.attributeDiffs;
  if (!attributeDiffs) return tree;

  for (const [attributePath, attributeDiff] of Object.entries(attributeDiffs)) {
    const attributePathSegments = attributePath.slice(1).split("/");

    // Do any fixups we want to the diff!
    // TODO fix this in the backend MV instead, and don't do this
    const diff = {
      new: fixAttributeSourceAndValue(attributeDiff.new),
      old: fixAttributeSourceAndValue(attributeDiff.old),
    } as AttributeDiff;

    if (!shouldIncludeDiff(attributePathSegments, diff)) continue;

    // Recursively create (or get) the element in the tree we want; then set the diff on it
    let child = tree;
    if (attributePath.length > 1) {
      for (const segment of attributePathSegments) {
        const path = `${child.path}/${segment}` as AttributePath;
        child.children ??= {};
        child.children[segment] ??= { path };
        child = child.children[segment] as AttributeDiffTree;
      }
    }
    child.diff = diff;
  }

  // Set the top level path correctly to / (kind of a special case)
  tree.path = "/";
  return tree;
}

function shouldIncludeDiff(
  attributePathSegments: string[],
  diff: AttributeDiff,
) {
  // If the values and sources are equal (which could happen in some cases on component
  // upgrade), don't show this diff.
  if (_.isEqual(diff.new, diff.old)) return false;

  if (!diff.old || !diff.new) {
    const { $source, $value } = diff.old ?? diff.new;
    // Don't show "uninteresting" default values (static values, or empty values from functions).
    if ($source.fromSchema) {
      if (_.isObject($value) && _.isEmpty($value)) return false;
      if (_.isArray($value) && _.isEmpty($value)) return false;
      if ($value === "" || $value === 0) return false;
      if ($value === undefined || $value === null) return false;
    }
    // Don't show new objects if they are fields of an object (otherwise we see a bunch of {}).
    // NOTE: If it's under a top-level path then we *know* it's a field of an object and can safely
    // not show it! We can't do the same for deeply nested fields until the MV tells us whether
    // the parent prop is an object or not (we can *only* avoid showing object fields).
    if (attributePathSegments.length <= 2) {
      if ("value" in $source && _.isObject($source.value)) return false;
    }
  }

  return true;
}

/**
 * Augment AttributeSourceAndValue with component name.
 *
 * This is where we put any fixups we need while working in the frontend; any changes here need
 * to move to the backend MV.
 */
function fixAttributeSourceAndValue(sourceAndValue?: AttributeSourceAndValue) {
  if (!sourceAndValue) return undefined;
  const { $source } = sourceAndValue;
  // Add componentName to $source
  if ("component" in $source) {
    const component = $source.component;
    const componentName = rawComponentList.value.find(
      (c) => c.id === component,
    )?.name;
    return {
      ...sourceAndValue,
      $source: {
        ...$source,
        componentName,
      },
    } as AttributeSourceAndValue;
  }
  // Make $value match $source (the only time it doesn't is object field defaults, which we don't want to show!)
  if ("value" in $source) {
    if (_.isObject($source.value)) {
      return {
        $source,
        $value: $source.value,
      };
    }
  }
  return sourceAndValue;
}

/** Overall (non-filtered) component counts for each diff status */
// const componentCounts = computed(() => {
//   const result = {
//     Added: 0,
//     Modified: 0,
//     None: 0,
//     Removed: 0,
//   };
//   for (const component of componentList.value) {
//     result[component.diffStatus] += 1;
//   }
//   return result;
// });

/** The currently-selected component data, including diffs */
const selectedComponent = computed(() =>
  componentList.value.find((c) => c.id === selectedComponentId.value),
);

const disableRevert = computed(
  () => selectedComponent.value?.diffStatus === "Removed",
);

// When absolutely anything in the selected component changes, or the selection itself changes,
// invalidate the audit logs query for that component.
watch(
  selectedComponent,
  (newSelectedComponent) => {
    if (newSelectedComponent) {
      queryClient.invalidateQueries({
        queryKey: key(EntityKind.AuditLogsForComponent, newSelectedComponent.id)
          .value,
      });
    }
  },
  { deep: true },
);

watch(
  () => ctx.onHead.value,
  (onHead) => {
    if (onHead) {
      exitReview();
    }
  },
);

// Watch for selected component changes and scroll it into view
watch(selectedComponentId, (newSelectedId) => {
  if (newSelectedId) {
    scrollSelectedComponentIntoView();
  }
});

const selectComponent = (componentId: ComponentId) => {
  selectedComponentId.value = componentId;
  // Push component ID to URL for deep linking
  router.push({
    name: route.name,
    params: route.params,
    query: {
      ...route.query,
      component: componentId,
    },
  });
};

const deselectComponent = () => {
  selectedComponentId.value = undefined;
  // Remove component from URL
  const { ...queryWithoutComponent } = route.query;
  router.push({
    name: route.name,
    params: route.params,
    query: queryWithoutComponent,
  });
};

const scrollSelectedComponentIntoView = async () => {
  if (!selectedComponentId.value || !componentListRef.value) return;

  await nextTick();

  // Find the selected component element
  const selectedElement = componentListRef.value.querySelector(
    `[data-component-id="${selectedComponentId.value}"]`,
  );

  if (selectedElement) {
    selectedElement.scrollIntoView({
      behavior: "smooth",
      block: "center",
      inline: "nearest",
    });
  }
};

const searchRef = ref<InstanceType<typeof SiSearch>>();
const componentListRef = ref<HTMLDivElement>();
const searchString = ref("");

/** Components, filtered by the search string */
const filteredComponentList = useComponentSearch(searchString, componentList);

// Watch for componentList changes and reinitialize from URL if needed
watch(
  filteredComponentList,
  (newList) => {
    if (newList && newList.length > 0 && !selectedComponentId.value) {
      initializeFromUrl();
    }
  },
  { immediate: true },
);

/** Added components, filtered by the search string */
const addedComponentList = computed(
  () =>
    filteredComponentList.value?.filter((c) => c.diffStatus === "Added") ?? [],
);
/** Modified components, filtered by the search string */
const modifiedComponentList = computed(
  () =>
    filteredComponentList.value?.filter((c) => c.diffStatus === "Modified") ??
    [],
);
/** Removed components, filtered by the search string */
const removedComponentList = computed(
  () =>
    filteredComponentList.value?.filter((c) => c.diffStatus === "Removed") ??
    [],
);

// Calculate action counts for the selected component, only including actions with count > 0
const { actionPrototypeViews, actionByPrototype } =
  useComponentActions(selectedComponent);
const actionCounts = computed(() => {
  const results: Record<string, { count: number; hasFailed: boolean }> = {};
  if (!selectedComponentId.value) return results;

  for (const actionPrototype of actionPrototypeViews.value) {
    const action = actionByPrototype.value[actionPrototype.id];
    if (action) {
      if (!action.componentId) continue;

      // Group Other actions with Manual
      let actionName = action.name;
      if (
        actionName.toLowerCase() === "other" ||
        action.kind?.toLowerCase() === "other"
      ) {
        actionName = "Manual";
      }

      if (!results[actionName]) {
        results[actionName] = { count: 0, hasFailed: false };
      }

      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      results[actionName]!.count += 1;

      // Track if any action in this group has failed
      if (action.state === ActionState.Failed) {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        results[actionName]!.hasFailed = true;
      }
    }
    // Remove the else block that creates entries with count: 0
  }

  // Filter out entries with count of 0
  return Object.fromEntries(
    Object.entries(results).filter(([_, value]) => value.count > 0),
  );
});

const exitReview = () => {
  router.push({
    name: "new-hotness",
  });
};

// Navigation logic for back/forward buttons
const currentComponentIndex = computed(() => {
  if (!selectedComponentId.value) return -1;
  return (
    filteredComponentList.value?.findIndex(
      (component) => component.id === selectedComponentId.value,
    ) ?? -1
  );
});

const canGoBack = computed(() => {
  // Always enabled if there are components (will select first if none selected, or go back if one is selected)
  return filteredComponentList.value?.length > 0;
});

const canGoForward = computed(() => {
  // Always enabled if there are components (will select first if none selected, or go forward if one is selected)
  return filteredComponentList.value?.length > 0;
});

const goToPreviousComponent = (e?: Event) => {
  e?.preventDefault();
  e?.stopPropagation();

  // Clear any text selection
  if (window.getSelection) {
    window.getSelection()?.removeAllRanges();
  }

  if (!selectedComponentId.value) {
    // No component selected - select the first one
    const firstComponent = filteredComponentList.value?.[0];
    if (firstComponent) {
      selectComponent(firstComponent.id);
    }
  } else if (currentComponentIndex.value > 0) {
    // Go to previous component
    const prevComponent =
      filteredComponentList.value?.[currentComponentIndex.value - 1];
    if (prevComponent) {
      selectComponent(prevComponent.id);
    }
  } else {
    // At first component, wrap around to last component
    const lastComponent =
      filteredComponentList.value?.[filteredComponentList.value.length - 1];
    if (lastComponent) {
      selectComponent(lastComponent.id);
    }
  }
};

const goToNextComponent = (e?: Event) => {
  e?.preventDefault();
  e?.stopPropagation();

  // Clear any text selection
  if (window.getSelection) {
    window.getSelection()?.removeAllRanges();
  }

  if (!selectedComponentId.value) {
    // No component selected - select the first one
    const firstComponent = filteredComponentList.value?.[0];
    if (firstComponent) {
      selectComponent(firstComponent.id);
    }
  } else if (
    currentComponentIndex.value <
    (filteredComponentList.value?.length ?? 0) - 1
  ) {
    // Go to next component
    const nextComponent =
      filteredComponentList.value?.[currentComponentIndex.value + 1];
    if (nextComponent) {
      selectComponent(nextComponent.id);
    }
  } else {
    // At last component, wrap around to first component
    const firstComponent = filteredComponentList.value?.[0];
    if (firstComponent) {
      selectComponent(firstComponent.id);
    }
  }
};

const controlUp = () => {
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];
  if (!selectedComponentId.value) {
    searchRef.value?.focusSearch();
    return;
  }

  const index = focusable.findIndex(
    (element) =>
      element.dataset.listItemComponentId === selectedComponentId.value,
  );

  if (index - 1 > -1) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const el = focusable[index - 1]!;
    el.focus();
    selectedComponentId.value = el.dataset.listItemComponentId;
  } else {
    deselectComponent();
    searchRef.value?.focusSearch();
  }
};
const controlDown = () => {
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];
  if (!selectedComponentId.value) {
    searchRef.value?.focusSearch();
    return;
  }

  const index = focusable.findIndex(
    (element) =>
      element.dataset.listItemComponentId === selectedComponentId.value,
  );

  if (index + 1 < focusable.length) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const el = focusable[index + 1]!;
    el.focus();
    selectedComponentId.value = el.dataset.listItemComponentId;
  } else {
    deselectComponent();
    searchRef.value?.focusSearch();
  }
};

const onEscape = () => {
  if (selectedComponentId.value) {
    deselectComponent();
  } else {
    exitReview();
  }
};
const onTab = (e: KeyDetails["Tab"]) => {
  e.preventDefault();
  if (e.shiftKey) {
    controlUp();
  } else {
    controlDown();
  }
};
const onSearchTab = (e: KeyboardEvent) => {
  e.preventDefault();
  if (e.shiftKey) {
    searchControl(true);
  } else {
    searchControl(false);
  }
};

const onArrowLeft = (e: KeyDetails["ArrowLeft"]) => {
  e.preventDefault();
  goToPreviousComponent();
};

const onArrowRight = (e: KeyDetails["ArrowRight"]) => {
  e.preventDefault();
  goToNextComponent();
};
const searchControl = (up: boolean) => {
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];
  if (up) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const el = focusable[focusable.length - 1]!;
    el.focus();
    selectedComponentId.value = el.dataset.listItemComponentId;
  } else {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const el = focusable[0]!;
    el.focus();
    selectedComponentId.value = el.dataset.listItemComponentId;
  }
};

const { restoreComponents } = useComponentDeletion(undefined, true);
/**
 * Status of restoring the current component
 *
 * This is undefined if no restore is happening or has happened for the current component.
 */
const restoreComponentStatus = ref<"inProgress" | "succeeded">();
// When you switch components, restoring gets set to `undefined` again so you can hit the button again.
watch(
  () => selectedComponent.value?.id,
  () => {
    restoreComponentStatus.value = undefined;
  },
);
/** Restore the current component */
const restoreComponent = async () => {
  if (restoreComponentStatus.value) return;
  restoreComponentStatus.value = "inProgress";
  try {
    if (!selectedComponent.value) return;
    await sleep(1000);
    const result = await restoreComponents([selectedComponent.value.id]);
    restoreComponentStatus.value = result.success ? "succeeded" : undefined;
  } catch (e) {
    restoreComponentStatus.value = undefined;
  }
};

onMounted(() => {
  keyEmitter.on("Escape", onEscape);
  keyEmitter.on("Tab", onTab);
  keyEmitter.on("ArrowUp", controlUp);
  keyEmitter.on("ArrowDown", controlDown);
  keyEmitter.on("ArrowLeft", onArrowLeft);
  keyEmitter.on("ArrowRight", onArrowRight);

  // Initialize component selection from URL
  initializeFromUrl();
});
onBeforeUnmount(() => {
  keyEmitter.off("Escape", onEscape);
  keyEmitter.off("Tab", onTab);
  keyEmitter.off("ArrowUp", controlUp);
  keyEmitter.off("ArrowDown", controlDown);
  keyEmitter.off("ArrowLeft", onArrowLeft);
  keyEmitter.off("ArrowRight", onArrowRight);
});

const noAVDiffs = computed(
  () =>
    // If no component is selected, this will return false
    selectedComponent.value &&
    !selectedComponent.value?.attributeDiffTree?.children?.si?.children?.name &&
    !selectedComponent.value?.attributeDiffTree?.children?.domain?.children &&
    !selectedComponent.value?.attributeDiffTree?.children?.secrets?.children,
);

const selectedComponentErased = computed(
  () =>
    selectedComponent.value?.diffStatus === "Removed" &&
    !selectedComponent.value.toDelete,
);

const goToComponentDetails = () => {
  if (!selectedComponentId.value || selectedComponentErased.value) return;

  router.push({
    name: "new-hotness-component",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      componentId: selectedComponentId.value,
    },
  });
};

const actionsRef = ref<InstanceType<typeof CollapsingFlexItem>>();
const actionsPanelRef = ref<InstanceType<typeof ActionsPanel>>();
const fixActionsPanelState = () => {
  if (selectedComponentId.value) {
    if (
      noAVDiffs.value &&
      actionsRef.value &&
      !actionsRef.value.openState.open.value
    ) {
      // Opens the actions panel if there are noAVDiffs
      actionsRef.value.openState.toggle();
    } else {
      nextTick(() => {
        if (
          actionsRef.value &&
          actionsPanelRef.value?.actionPrototypeViews &&
          actionsPanelRef.value?.actionPrototypeViews.length === 0 &&
          actionsRef.value.openState.open.value
        ) {
          // Closes the actions panel if there are AVDiffs and no actions
          actionsRef.value.openState.toggle();
        }
      });
    }
  }
};

watch(selectedComponentId, () => {
  fixActionsPanelState();
});
</script>

<style lang="css" scoped>
section.grid.review {
  grid-template-columns: minmax(0, 25%) minmax(0, 50%) minmax(0, 25%);
  grid-template-rows: 100%;
  grid-template-areas: "left main right";
}
div.main {
  grid-area: "main";
}
div.left {
  grid-area: "left";
}
div.right {
  grid-area: "right";
}
</style>
