<template>
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
  <section v-else class="grid review w-full h-full">
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
      <div class="flex flex-col gap-xs flex-grow">
        <CollapsingFlexItem headerTextSize="sm" open>
          <template #header>Created</template>
          <template #headerIcons>
            <PillCounter :count="addedComponentList.length" size="sm" />
          </template>
          <div class="flex flex-col gap-xs p-xs w-full h-full">
            <EmptyState
              v-if="componentCounts.Added === 0"
              icon="component"
              text="No components added"
              class="p-sm"
            />
            <EmptyState
              v-else-if="addedComponentList.length === 0"
              icon="component"
              text="No added components match your search"
              class="p-sm"
            />
            <ComponentListItem
              v-for="component in addedComponentList"
              :key="component.id"
              :component="component"
              :selected="component.id === selectedComponentId"
              @click="selectComponent(component.id)"
            />
          </div>
        </CollapsingFlexItem>
        <CollapsingFlexItem headerTextSize="sm" open>
          <template #header>Changed</template>
          <template #headerIcons>
            <PillCounter :count="modifiedComponentList.length" size="sm" />
          </template>
          <div class="flex flex-col gap-xs p-xs w-full h-full">
            <EmptyState
              v-if="componentCounts.Modified === 0"
              icon="diff"
              text="No components changed"
              class="p-sm"
            />
            <EmptyState
              v-else-if="modifiedComponentList.length === 0"
              icon="diff"
              text="No changed components match your search"
              class="p-sm"
            />
            <ComponentListItem
              v-for="component in modifiedComponentList"
              :key="component.id"
              :component="component"
              :selected="component.id === selectedComponentId"
              @click="selectComponent(component.id)"
            />
          </div>
        </CollapsingFlexItem>
        <CollapsingFlexItem headerTextSize="sm" open>
          <template #header>Removed</template>
          <template #headerIcons>
            <PillCounter :count="removedComponentList.length" size="sm" />
          </template>
          <div class="flex flex-col gap-xs p-xs w-full h-full">
            <EmptyState
              v-if="componentCounts.Removed === 0"
              icon="trash"
              text="No components deleted"
              class="p-sm"
            />
            <EmptyState
              v-else-if="removedComponentList.length === 0"
              icon="trash"
              text="No deleted components match your search"
              class="p-sm"
            />
            <ComponentListItem
              v-for="component in removedComponentList"
              :key="component.id"
              :component="component"
              :selected="component.id === selectedComponentId"
              @click="selectComponent(component.id)"
            />
          </div>
        </CollapsingFlexItem>
      </div>
    </div>
    <div class="main flex flex-col gap-xs m-xs">
      <CollapsingFlexItem
        v-if="selectedComponentId && selectedComponent"
        disableCollapse
      >
        <template #header>
          <div>{{ selectedComponent.schemaName }}</div>
          <div>"{{ selectedComponent.name }}"</div>
        </template>

        <div class="flex flex-col gap-xs p-xs">
          <!-- Show /si/name-->
          <ReviewAttributeItem
            v-if="
              selectedComponent.attributeDiffTree?.children?.si?.children?.name
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
      <CollapsingFlexItem>
        <template #header> Actions </template>
      </CollapsingFlexItem>
    </div>
    <div class="right flex flex-col p-xs">
      <CollapsingFlexItem open>
        <template #header>Component History</template>
        <template v-if="selectedComponentId">
          <ComponentHistory :componentId="selectedComponentId"
        /></template>
        <EmptyState
          v-else
          class="p-lg"
          icon="component"
          text="No component selected"
          secondaryText="Select a component to see its history"
        />
      </CollapsingFlexItem>
      <CollapsingFlexItem open>
        <template #header>Diff</template>
        <CodeViewer
          v-if="selectedComponent"
          :title="`${selectedComponent.name}: ${selectedComponent.schemaName}`"
          :code="changedComponentData[selectedComponent.id]?.resourceDiff.diff"
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
</template>

<script setup lang="ts">
import { useQueries, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import clsx from "clsx";
import {
  PillCounter,
  SiSearch,
  themeClasses,
  VButton,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import * as _ from "lodash-es";
import {
  bifrost,
  bifrostList,
  useMakeArgs,
  useMakeArgsForHead,
  useMakeKey,
  useMakeKeyForHead,
} from "@/store/realtime/heimdall";
import {
  AttributeDiff,
  AttributeSourceAndValue,
  BifrostComponent,
  ComponentDiff,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import ComponentListItem from "./ComponentListItem.vue";
import { useContext } from "./logic_composables/context";
import EmptyState from "./EmptyState.vue";
import ComponentHistory from "./ComponentHistory.vue";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import ReviewAttributeItem from "./ReviewAttributeItem.vue";
import { KeyDetails, keyEmitter } from "./logic_composables/emitters";
import { useComponentSearch } from "./logic_composables/search";

const router = useRouter();
const ctx = useContext();
const queryClient = useQueryClient();
queryClient.setDefaultOptions({ queries: { staleTime: Infinity } });

// const changeSetName = computed(() => ctx.changeSet.value?.name);
const selectedComponentId = ref<ComponentId>();

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

// Get component list from head so we can tell what components have been removed
const headKey = useMakeKeyForHead();
const headArgs = useMakeArgsForHead();

const headComponentListQuery = useQuery({
  queryKey: headKey(EntityKind.ComponentList),
  queryFn: async () =>
    await bifrostList<ComponentInList[]>(headArgs(EntityKind.ComponentList)),
});
const headComponentList = computed(
  () => headComponentListQuery.data.value ?? [],
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

  // Add any head components that aren't in the list as "removed"
  for (const headComponent of headComponentList.value) {
    if (!result.find((c) => c.id === headComponent.id)) {
      result.push({
        ...headComponent,
        diffStatus: "Removed" as const,
      });
    }
  }

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
  return rawComponentList.value
    .map((component) => {
      // TEMPORARY: "fix" issues in the MV so we can do better testing / display in the short term.
      // Ultimately, we will fix the MV instead.
      const componentDiff = componentDiffs[component.id];
      const attributeDiffTree = toAttributeDiffTree(componentDiff);
      return {
        ...component,
        diffStatus: componentDiff?.diffStatus ?? component.diffStatus,
        componentDiff,
        attributeDiffTree,
      };
    })
    .filter(
      (component) =>
        component.attributeDiffTree.children &&
        Object.keys(component.attributeDiffTree.children).length > 0,
    );
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
  // upgrade), don't add this to the tree
  if (_.isEqual(diff.new, diff.old)) return false;

  // If this is an empty default (schema unset/empty container) on an added/removed thing, don't
  // include it in the diff
  if (!diff.new || !diff.old) {
    const singleDiff = diff.new ?? diff.old;
    if (singleDiff.$value === undefined && singleDiff.$source.fromSchema)
      return false;
    if (singleDiff.$source.fromSchema) {
      if (singleDiff.$value === undefined) return false;
      if (Array.isArray(singleDiff.$value) && singleDiff.$value.length === 0)
        return false;
      // TODO empty object
    }
  }

  return true;
}

function fixAttributeSourceAndValue(sourceAndValue?: AttributeSourceAndValue) {
  if (!sourceAndValue) return undefined;
  const { $source } = sourceAndValue;
  if ($source.prototype !== undefined) {
    // subscription to /domain/region on Region changed (01K2N2CFSAE0PPHNJ31SKP97HH)
    const { prototype, ...otherFields } = $source;
    const match = prototype.match(/^subscription to (\/.+) on .+ \((\w+)\)/);
    if (match) {
      const [, path, component] = match;
      if (path && component) {
        const componentName = rawComponentList.value.find(
          (c) => c.id === component,
        )?.name;
        return {
          ...sourceAndValue,
          $source: {
            component,
            componentName,
            path,
            ...otherFields,
          },
        } as AttributeSourceAndValue;
      }
    }
  }
  return sourceAndValue;
}

/** Overall (non-filtered) component counts for each diff status */
const componentCounts = computed(() => {
  const result = {
    Added: 0,
    Modified: 0,
    None: 0,
    Removed: 0,
  };
  for (const component of componentList.value) {
    result[component.diffStatus] += 1;
  }
  return result;
});

// Grab the Component MV for any changed components (for the text diff)
// NOTE we should probably just dump this data into the ComponentDiff instead, but that'll be
// a factoring turn.
const changedComponentDataQueries = useQueries({
  queries: computed(() => {
    return componentList.value
      .filter((component) => component.diffStatus !== "None")
      .map((component) => ({
        queryKey: key(EntityKind.Component, component.id),
        queryFn: async () =>
          await bifrost<BifrostComponent>(
            args(EntityKind.Component, component.id),
          ),
      }));
  }),
});

/** Component MV for any changed components */
const changedComponentData = computed<{
  [id in ComponentId]?: BifrostComponent;
}>(() =>
  Object.fromEntries(
    changedComponentDataQueries.value.map(
      (component) => [component.data?.id, component.data] as const,
    ),
  ),
);

/** The currently-selected component data, including diffs */
const selectedComponent = computed(() => {
  if (!selectedComponentId.value) {
    return undefined;
  }
  return {
    id: selectedComponentId.value,
    ...componentList.value.find(
      (component) => component.id === selectedComponentId.value,
    ),
    details: changedComponentData.value[selectedComponentId.value],
  };
});

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

const selectComponent = (componentId: ComponentId) => {
  selectedComponentId.value = componentId;
};

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");

/** Components, filtered by the search string */
const filteredComponentList = useComponentSearch(searchString, componentList);
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

const exitReview = () => {
  router.push({
    name: "new-hotness",
  });
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
    selectedComponentId.value = undefined;
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
    selectedComponentId.value = undefined;
    searchRef.value?.focusSearch();
  }
};

const onEscape = () => {
  if (selectedComponentId.value) {
    selectedComponentId.value = undefined;
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
});
onBeforeUnmount(() => {
  keyEmitter.off("Escape", onEscape);
  keyEmitter.on("Tab", onTab);
  keyEmitter.on("ArrowUp", controlUp);
  keyEmitter.on("ArrowDown", controlDown);
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
