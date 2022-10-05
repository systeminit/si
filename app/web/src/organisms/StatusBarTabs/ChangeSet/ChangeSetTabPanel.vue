<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <StatusBarTabPanelComponentList
      :component-list="componentsList"
      :selected-filter="selectedFilter"
      :filter-options="filterOptions"
      @filter="changeSelectedFilter"
    />

    <!-- Selected component view -->
    <div
      v-if="selectedComponent"
      class="w-full h-full flex flex-col bg-shade-100"
    >
      <div
        v-if="selectedComponent.changeStatus === 'added'"
        class="w-full h-full p-2 first-letter:overflow-hidden flex flex-row flex-wrap"
      >
        <div
          class="w-full h-fit max-h-full p-2 overflow-hidden bg-neutral-800 rounded flex flex-row"
        >
          <CodeViewer
            font-size="13px"
            class="text-neutral-50 mx-2"
            :code="codeRecord['Current']"
            :code-language="getCodeLanguage('Current')"
            force-theme="dark"
          >
            <template #title>
              <span class="text-lg">Current</span>
            </template>
          </CodeViewer>
        </div>
      </div>

      <div
        v-else-if="selectedComponent.changeStatus === 'deleted'"
        class="flex flex-row items-center text-center w-full h-full"
      >
        <p class="w-full text-3xl text-destructive-300">Component Deleted</p>
      </div>

      <div
        v-else-if="selectedComponent.changeStatus === 'modified'"
        class="overflow-x-hidden flex flex-row flex-wrap p-2"
      >
        <div
          v-for="title in ['Current', 'Diff']"
          :key="title"
          class="h-fit max-h-full lg:basis-1/2 overflow-hidden flex flex-row p-2"
        >
          <div
            class="w-full shrink grow bg-neutral-800 rounded p-1 flex flex-row"
          >
            <CodeViewer
              font-size="13px"
              class="text-neutral-50 mx-2"
              :code="codeRecord[title]"
              force-theme="dark"
              :code-language="getCodeLanguage(title)"
            >
              <template #title>
                <span class="text-lg">{{ title }}</span>
              </template>
            </CodeViewer>
          </div>
        </div>
      </div>
    </div>
    <div
      v-else
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { from, switchMap, map, combineLatest } from "rxjs";
import { computed, ref } from "vue";
import { refFrom, fromRef } from "vuse-rx";
import _ from "lodash";
import { GlobalErrorService } from "@/service/global_error";
import CodeViewer from "@/organisms/CodeViewer.vue";
import { ComponentService } from "@/service/component";
import { ComponentDiff } from "@/api/sdf/dal/component";
import StatusBarTabPanelComponentList, {
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { useComponentsStore } from "@/store/components.store";

const defaultFilterOption = {
  value: "all",
  title: "Show All",
};
const filterOptions: FilterOption[] = [
  defaultFilterOption,
  {
    value: "added",
    title: "Added",
  },
  {
    value: "modified",
    title: "Modified",
  },
  {
    value: "deleted",
    title: "Deleted",
  },
];

const selectedFilter = ref<FilterOption>(defaultFilterOption);
const changeSelectedFilter = (newFilter: FilterOption) => {
  selectedFilter.value = newFilter;
};

const componentsStore = useComponentsStore();

// first filter down all components to only those changed
const changedComponents = computed(() =>
  _.filter(componentsStore.allComponents, (c) => !!c.changeStatus),
);
// now filter based on selected filter (added/modified/deleted)
const filteredChangedComponents = computed(() => {
  if (selectedFilter.value.value === "all") return changedComponents.value;
  return _.filter(
    changedComponents.value,
    (c) => c.changeStatus === selectedFilter.value.value,
  );
});
// convert into format needed by StatusBarTabPanelComponentList
const componentsList = computed(() =>
  _.map(filteredChangedComponents.value, (c) => ({
    id: c.id,
    name: c.displayName,
    status: c.changeStatus,
  })),
);

const selectedComponentId = computed(() => componentsStore.selectedComponentId);
const selectedComponent = computed(() =>
  _.find(
    filteredChangedComponents.value,
    (c) => c.id === selectedComponentId.value,
  ),
);

// FIXME(nick): we should be using the "unknown" language if there's no code once mode switching is reactive.
const codeRecord = computed((): Record<string, string> => {
  const code = {
    Diff: "# Waiting for component diff...",
    Current: "# Waiting for component diff...",
  };
  if (componentDiff.value) {
    if (componentDiff.value.diffs.length > 0) {
      // FIXME(nick): allow for multiple diffs.
      code.Diff = componentDiff.value.diffs[0].code ?? "# No code found";
    } else {
      // This should never be read, but is here just in case.
      code.Diff = "# No diff found";
    }
    code.Current = componentDiff.value.current.code ?? "# No code found";
  }
  return code;
});

// FIXME(nick): remove this once reactivity is fixed.
const getCodeLanguage = (title: string) => {
  if (title === "Current") {
    return "json";
  }
  // Default to diff.
  return "diff";
};

const componentDiff = refFrom<ComponentDiff | null>(
  combineLatest([fromRef(selectedComponentId)]).pipe(
    switchMap(([selectedComponentId]) => {
      if (selectedComponentId) {
        return ComponentService.getDiff({
          componentId: selectedComponentId,
        });
      }
      return from([null]);
    }),
    map((response) => {
      if (response) {
        if (response.error) {
          GlobalErrorService.set(response);
          return null;
        } else {
          return response.componentDiff;
        }
      }
      return null;
    }),
  ),
);
</script>
