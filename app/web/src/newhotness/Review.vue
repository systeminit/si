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
            <PillCounter :count="addedComponentListFiltered.length" size="sm" />
          </template>
          <div class="flex flex-col gap-xs p-xs w-full h-full">
            <EmptyState
              v-if="addedComponentList.length === 0"
              icon="component"
              text="No components added"
              class="p-sm"
            />
            <EmptyState
              v-else-if="addedComponentListFiltered.length === 0"
              icon="component"
              text="No added components match your search"
              class="p-sm"
            />
            <ComponentListItem
              v-for="component in addedComponentListFiltered"
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
            <PillCounter
              :count="modifiedComponentListFiltered.length"
              size="sm"
            />
          </template>
          <div class="flex flex-col gap-xs p-xs w-full h-full">
            <EmptyState
              v-if="modifiedComponentList.length === 0"
              icon="diff"
              text="No components changed"
              class="p-sm"
            />
            <EmptyState
              v-else-if="modifiedComponentListFiltered.length === 0"
              icon="diff"
              text="No changed components match your search"
              class="p-sm"
            />
            <ComponentListItem
              v-for="component in modifiedComponentListFiltered"
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
            <PillCounter
              :count="removedComponentListFiltered.length"
              size="sm"
            />
          </template>
          <div class="flex flex-col gap-xs p-xs w-full h-full">
            <EmptyState
              v-if="removedComponentList.length === 0"
              icon="trash"
              text="No components deleted"
              class="p-sm"
            />
            <EmptyState
              v-else-if="removedComponentListFiltered.length === 0"
              icon="trash"
              text="No deleted components match your search"
              class="p-sm"
            />
            <ComponentListItem
              v-for="component in removedComponentListFiltered"
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
      <CollapsingFlexItem v-if="selectedComponent" disableCollapse>
        <template #header>
          <div>{{ selectedComponent.schemaName }}</div>
          <div>"{{ selectedComponent.name }}"</div>
        </template>

        <div v-if="selectedComponent.diff" class="flex flex-col gap-xs p-xs">
          <ReviewAttributeItem
            v-for="(diff, path) in selectedComponentDisplayDiffs"
            :key="path"
            :path="path"
            :diff="diff"
          />
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
        <template v-if="selectedComponent">
          <EmptyState
            class="p-lg"
            icon="component"
            text="No History"
            secondaryText="This section is not done yet"
          />
        </template>
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
import { useQueries, useQuery } from "@tanstack/vue-query";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import clsx from "clsx";
import {
  PillCounter,
  SiSearch,
  themeClasses,
  VButton,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
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
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import ReviewAttributeItem from "./ReviewAttributeItem.vue";
import { KeyDetails, keyEmitter } from "./logic_composables/emitters";
import { useComponentSearch } from "./logic_composables/search";

const router = useRouter();
const ctx = useContext();

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

const componentList = computed(() => {
  return changeSetComponentList.value.concat(
    headComponentList.value
      .filter(
        (component) =>
          !changeSetComponentList.value.some((c) => c.id === component.id),
      )
      .map((component) => ({ ...component, diffStatus: "Removed" as const })),
  );
});

/** Complete attribute-by-attribute diff of every component */
const componentDiffQueries = useQueries({
  queries: computed(() =>
    componentList.value.map((component) => ({
      queryKey: key(EntityKind.ComponentDiff, component.id),
      queryFn: async () =>
        await bifrost<ComponentDiff>(
          args(EntityKind.ComponentDiff, component.id),
        ),
    })),
  ),
});
/** Complete attribute-by-attribute diff of every component */
const componentDiffs = computed<Record<ComponentId, ComponentDiff>>(() =>
  Object.fromEntries(
    componentDiffQueries.value
      .map((query) => [query.data?.id, query.data] as const)
      .filter(([id, diff]) => id && diff),
  ),
);

/**
 * Component list, for *changed* components.
 *
 * NOTE: this merges in the diffStatus from the ComponentDiff MV, which is more accurate. At some
 * point we will want to modify the ComponentList MV to use the more accurate diff, so we don't have
 * to pull every single ComponentDiff MV to figure it out.
 */
const changedComponentList = computed(() =>
  componentList.value
    .map((component) => ({
      ...component,
      diffStatus:
        componentDiffs.value[component.id]?.diffStatus ?? component.diffStatus,
    }))
    .filter((component) => component.diffStatus !== "None"),
);

// Grab the Component MV for any changed components (for the text diff)
// NOTE we should probably just dump this data into the ComponentDiff instead, but that'll be
// a factoring turn.
const changedComponentDataQueries = useQueries({
  queries: computed(() =>
    changedComponentList.value.map((component) => ({
      queryKey: key(EntityKind.Component, component.id),
      queryFn: async () =>
        await bifrost<BifrostComponent>(
          args(EntityKind.Component, component.id),
        ),
    })),
  ),
});
const changedComponentData = computed<Record<ComponentId, BifrostComponent>>(
  () =>
    Object.fromEntries(
      changedComponentDataQueries.value
        .map((component) => [component.data?.id, component.data] as const)
        .filter(([id, diff]) => id && diff),
    ),
);

/** ComponentList of added components only */
const addedComponentList = computed(() =>
  changedComponentList.value.filter(
    (component) => component.diffStatus === "Added",
  ),
);

/** ComponentList of modified components only */
const modifiedComponentList = computed(() =>
  changedComponentList.value.filter(
    (component) => component.diffStatus === "Modified",
  ),
);

/** ComponentList of modified components only */
const removedComponentList = computed(() =>
  changedComponentList.value.filter(
    (component) => component.diffStatus === "Removed",
  ),
);

const selectComponent = (componentId: ComponentId) => {
  selectedComponentId.value = componentId;
};

/** The currently-selected component data, including diffs */
const selectedComponent = computed(() => {
  if (!selectedComponentId.value) {
    return undefined;
  }
  return {
    id: selectedComponentId.value,
    ...changedComponentList.value.find(
      (component) => component.id === selectedComponentId.value,
    ),
    details: componentDiffs.value[selectedComponentId.value],
    diff: componentDiffs.value[selectedComponentId.value],
  };
});

const selectedComponentDisplayDiffs = computed(() => {
  if (selectedComponent.value?.diff?.attributeDiffs) {
    const entries = Object.entries(selectedComponent.value.diff.attributeDiffs);
    const output: [AttributePath, AttributeDiff][] = [];

    entries.forEach((entry) => {
      const path = entry[0];
      if (path.startsWith("/domain") || path === "/si/name") {
        output.push(entry);
      }
    });

    return Object.fromEntries(output) as Record<AttributePath, AttributeDiff>;
  } else {
    return {};
  }
});

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");
const addedComponentListFiltered = useComponentSearch(
  searchString,
  addedComponentList,
);
const modifiedComponentListFiltered = useComponentSearch(
  searchString,
  modifiedComponentList,
);
const removedComponentListFiltered = useComponentSearch(
  searchString,
  removedComponentList,
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
