<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <StatusBarTabPanelComponentList
      :component-list="list"
      :selected-filter="selectedFilter"
      :filter-options="filterOptions"
      @filter="changeSelectedFilter"
    />

    <!-- Selected component view -->
    <div
      v-if="selectedComponentId && selectedComponentStatsGroup"
      class="w-full h-full flex flex-col bg-shade-100"
    >
      <div
        v-if="!filterMatchesSelectedComponentStatus"
        class="flex flex-row items-center text-center w-full h-full bg-shade-100"
      >
        <p class="w-full text-3xl text-neutral-500">
          Selected Component Has Not Been {{ selectedFilter.title }}
        </p>
      </div>

      <div
        v-else-if="selectedComponentStatsGroup.componentStatus === 'added'"
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
        v-else-if="selectedComponentStatsGroup.componentStatus === 'deleted'"
        class="flex flex-row items-center text-center w-full h-full"
      >
        <p class="w-full text-3xl text-destructive-300">Component Deleted</p>
      </div>

      <div
        v-else-if="selectedComponentStatsGroup.componentStatus === 'modified'"
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
import { refFrom, fromRef, untilUnmounted } from "vuse-rx";
import _ from "lodash";
import { ComponentStats } from "@/api/sdf/dal/change_set";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import CodeViewer from "@/organisms/CodeViewer.vue";
import { ComponentService } from "@/service/component";
import { ComponentDiff } from "@/api/sdf/dal/component";
import { SelectionService } from "@/service/selection";
import StatusBarTabPanelComponentList, {
  ComponentListItem,
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";

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

// Only display individual component data if the filter is valid for the component status.
// The filter is valid in two scenarios:
// 1) If a component is selected _and_ the selected filter value is "all"
// 2) If a component is selected _and_ its status matches the selected filter value
const filterMatchesSelectedComponentStatus = computed((): boolean => {
  if (selectedComponentStatsGroup.value) {
    if (
      selectedFilter.value.value === "all" ||
      selectedComponentStatsGroup.value.componentStatus ===
        selectedFilter.value.value
    ) {
      return true;
    }
    // Fall back to false if we have a selected component, but it doesn't pass the condition.
  }
  return false;
});

const list = computed((): ComponentListItem[] => {
  if (!stats.value) return [];

  let list = [];
  for (const statsGroup of stats.value.stats) {
    list.push({
      id: statsGroup.componentId,
      name: statsGroup.componentName,
      status: statsGroup.componentStatus,
    });
  }

  // Filter the results if a filter has been selected.
  if (
    selectedFilter.value &&
    selectedFilter.value.value !== defaultFilterOption.value
  ) {
    list = list.filter((item) => item.status === selectedFilter.value.value);
  }

  return list;
});

const stats = ref<ComponentStats>({ stats: [] });

untilUnmounted(ChangeSetService.getStats()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
  } else {
    stats.value = response.componentStats;
  }
});

const selectedComponentId = SelectionService.useSelectedComponentId();
const localSelectedComponentId$ = fromRef(selectedComponentId);
const selectedComponentStatsGroup = computed(() => {
  return _.find(
    stats.value.stats,
    (statsGroup) => statsGroup.componentId === selectedComponentId.value,
  );
});

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
  combineLatest([localSelectedComponentId$]).pipe(
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
