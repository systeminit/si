<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <StatusBarTabPanelComponentList
      :component-list="componentsList"
      :selected-filter="selectedFilter"
      :filter-options="filterOptions"
      @filter="changeSelectedFilter"
    >
      <template #icon="{ component }">
        <StatusIndicatorIcon
          type="change"
          :status="changeStatusByComponentId[component.id]"
        />
      </template>
    </StatusBarTabPanelComponentList>

    <!-- Selected component view -->

    <div
      v-if="!selectedComponent"
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>

    <div v-else class="w-full h-full flex flex-col bg-shade-100">
      <template v-if="diffReqStatus.isPending"
        >Loading component code...</template
      >
      <template v-else-if="diffReqStatus.isError">
        <ErrorMessage :request-status="diffReqStatus" />
      </template>
      <template v-else-if="diffReqStatus.isSuccess && selectedComponentDiff">
        <div
          v-if="selectedComponent.changeStatus === 'deleted'"
          class="flex flex-row items-center text-center w-full h-full"
        >
          <p class="w-full text-3xl text-destructive-300">Component Deleted</p>
        </div>

        <div
          v-else
          class="w-full h-full p-2 first-letter:overflow-hidden flex flex-row flex-wrap"
        >
          <div
            class="w-full h-fit max-h-full p-2 overflow-hidden bg-neutral-800 rounded flex flex-row"
          >
            <CodeViewer
              font-size="13px"
              class="text-neutral-50 mx-2"
              :code="selectedComponentDiff.current.code"
              :code-language="selectedComponentDiff.current.language"
            >
              <template #title>
                <span class="text-lg">Current</span>
              </template>
            </CodeViewer>

            <template v-if="selectedComponent.changeStatus === 'modified'">
              <!-- what to do about multiple diffs? -->
              <CodeViewer
                font-size="13px"
                class="text-neutral-50 mx-2"
                :code="selectedComponentDiff.diffs[0].code"
                :code-language="selectedComponentDiff.diffs[0].language"
              >
                <template #title>
                  <span class="text-lg">Diff</span>
                </template>
              </CodeViewer>
            </template>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import _ from "lodash";
import CodeViewer from "@/organisms/CodeViewer.vue";
import StatusBarTabPanelComponentList, {
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { useComponentsStore } from "@/store/components.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";

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
const changeSetsStore = useChangeSetsStore();

const changeStatusByComponentId = computed(
  () => componentsStore.componentChangeStatusById,
);
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

const selectedComponentDiff = computed(
  () => componentsStore.selectedComponentDiff,
);

const diffReqStatus = componentsStore.getRequestStatus(
  "FETCH_COMPONENT_DIFF",
  selectedComponentId,
);

watch(
  [selectedComponentId, () => changeSetsStore.selectedChangeSetWritten],
  () => {
    if (!selectedComponentId.value) return;
    componentsStore.FETCH_COMPONENT_DIFF(selectedComponentId.value);
  },
  { immediate: true },
);
</script>
