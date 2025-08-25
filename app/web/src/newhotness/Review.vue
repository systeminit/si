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
      <IconButton
        tooltip="Close (Esc)"
        tooltipPlacement="top"
        class="border-0 mr-2em"
        icon="x"
        size="sm"
        iconIdleTone="shade"
        iconTone="shade"
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
        <VButton
          label="Previous"
          size="sm"
          tone="neutral"
          :disabled="!canGoBack"
          @click.stop.prevent="goToPreviousComponent"
        >
          <template #icon>
            <div class="border border-neutral-400 rounded px-2xs py-2xs mr-2xs">
              <Icon name="arrow--left" size="xs" />
            </div>
          </template>
        </VButton>
        <VButton
          label="Next"
          size="sm"
          :class="
            clsx(
              '!text-sm !border !cursor-pointer !px-xs',
              themeClasses(
                '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
              ),
            )
          "
          :disabled="!canGoForward"
          @click.stop.prevent="goToNextComponent"
        >
          <template #iconRight>
            <div class="border border-action-200 rounded px-2xs py-2xs ml-2xs">
              <Icon name="arrow--right" size="xs" />
            </div>
          </template>
        </VButton>
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
      <VButton
        label="Exit"
        tone="neutral"
        icon="chevron--left"
        @click="exitReview"
      />
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
            <TruncateWithTooltip
              :class="
                clsx(
                  'py-2xs max-w-fit flex-1',
                  themeClasses('text-neutral-800', 'text-neutral-100'),
                )
              "
            >
              {{ selectedComponent.schemaName }}
            </TruncateWithTooltip>
            <TruncateWithTooltip
              :class="
                clsx(
                  'py-2xs max-w-fit flex-1',
                  themeClasses('text-neutral-600', 'text-neutral-400'),
                )
              "
            >
              ({{ selectedComponent.name }})
            </TruncateWithTooltip>
          </template>

          <div class="flex flex-col gap-sm px-sm py-sm">
            <div
              v-if="selectedComponent.toDelete"
              :class="
                clsx(
                  'flex flex-col gap-xs p-sm border text-sm',
                  themeClasses(
                    'text-neutral-800 border-neutral-400 bg-neutral-100',
                    'text-neutral-100 border-neutral-600 bg-neutral-900',
                  ),
                )
              "
            >
              This component is set to be deleted from HEAD once the change set
              is applied, all values will be cleaned.
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
                secret
              />
            </template>
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
            <ActionsPanel :component="selectedComponent" />
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
  VButton,
  IconButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import * as _ from "lodash-es";
import {
  bifrost,
  bifrostList,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import {
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
  // Add diffs to each component in the change set, and include "removed"
  const componentDiffs: { [id in ComponentId]?: ComponentDiff } =
    Object.fromEntries(
      componentDiffQueries.value.map(
        (query) => [query.data?.id, query.data] as const,
      ),
    );
  const mapped = rawComponentList.value
    .map((component) => {
      const componentDiff = componentDiffs[component.id];
      const attributeDiffTree = toAttributeDiffTree(componentDiff);
      const diffStatus = component.toDelete
        ? "Removed"
        : componentDiff?.diffStatus ?? component.diffStatus;
      return {
        ...component,
        diffStatus,
        componentDiff,
        attributeDiffTree,
      };
    })
    .filter((component) => component.diffStatus !== "None");
  if (erasedComponents.data.value?.erased) {
    for (const { diff, component } of Object.values(
      erasedComponents.data.value?.erased,
    )) {
      const attributeDiffTree = toAttributeDiffTree(diff);
      mapped.push({
        ...component,
        diffStatus: "Removed",
        componentDiff: diff,
        attributeDiffTree,
      });
    }
  }
  return mapped;
});

/** A tree version of AttributeDiff:
 * {
 *   children: {
 *     domain: {
 *       children: {
 *         SubnetIds: {
 *           children: {
 *             0: { diff: ... }
 *           }
 *           // parents may or may not have a diff! Especially if they have a *source* difference
 *           diff: {...},
 *         },
 *         extra: {
 *           children: {
 *             Region: { diff: {} }
 *           }
 *         }
 *       }
 *     }
 *   }
 * }
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
    // Do any fixups we want to the diff!
    // TODO fix this in the backend MV instead, and don't do this
    const diff = {
      new: fixAttributeSourceAndValue(attributeDiff.new),
      old: fixAttributeSourceAndValue(attributeDiff.old),
    } as AttributeDiff;

    if (!shouldIncludeDiff(diff)) continue;

    // Recursively create (or get) the element in the tree we want; then set the diff on it
    let child = tree;
    if (attributePath.length > 1) {
      for (const segment of attributePath.slice(1).split("/")) {
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

function shouldIncludeDiff(diff: AttributeDiff) {
  // If the values and sources are equal (which could happen in some cases on component
  // upgrade), don't show this diff.
  if (_.isEqual(diff.new, diff.old)) return false;

  if (!diff.old || !diff.new) {
    const { $source, $value } = diff.old ?? diff.new;
    // Don't show "uninteresting" (empty) default values.
    if ($source.fromSchema) {
      if (!$value || _.isEmpty($value)) return false;
    }
  }

  return true;
}

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
