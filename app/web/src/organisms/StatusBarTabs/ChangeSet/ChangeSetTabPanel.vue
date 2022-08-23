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
        v-if="selectedComponentStatsGroup.componentStatus === 'added'"
        class="overflow-y-auto flex flex-row flex-wrap"
      >
        <div class="basis-full overflow-hidden pr-10 pl-1 pt-2">
          <!-- FIXME(nick): make code viewer height malleable if the status bar no longer a fixed size.
               1024 is the "min-width" for "lg" in tailwind, so we (maybe) should use it to detect if we need to add a height. -->
          <CodeViewer
            font-size="13px"
            height="250px"
            :component-id="selectedComponentStatsGroup.componentId"
            class="text-neutral-50 mx-5"
            :code="codeRecord['Current']"
            force-theme="dark"
            :code-language="getCodeLanguage('Current')"
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
        class="overflow-y-auto flex flex-row flex-wrap"
      >
        <div
          v-for="title in ['Current', 'Diff']"
          :key="title"
          class="basis-full lg:basis-1/2 overflow-hidden pr-10 pl-1 pt-2"
        >
          <!-- FIXME(nick): make code viewer height malleable if the status bar no longer a fixed size.
               1024 is the "min-width" for "lg" in tailwind, so we (maybe) should use it to detect if we need to add a height. -->
          <CodeViewer
            font-size="13px"
            height="250px"
            :component-id="selectedComponentStatsGroup.componentId"
            class="text-neutral-50 mx-5"
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
    <div
      v-else
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ComponentStats } from "@/api/sdf/dal/change_set";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { from, switchMap, map } from "rxjs";
import { computed, ref } from "vue";
import { refFrom, fromRef, untilUnmounted } from "vuse-rx";
import CodeViewer from "@/organisms/CodeViewer.vue";
import { combineLatest } from "rxjs";
import { ComponentService } from "@/service/component";
import { ComponentDiff } from "@/api/sdf/dal/component";
import _ from "lodash";
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
      code["Diff"] = componentDiff.value.diffs[0].code ?? "# No code found";
    } else {
      // This should never be read, but is here just in case.
      code["Diff"] = "# No diff found";
    }
    code["Current"] = componentDiff.value.current.code ?? "# No code found";
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
