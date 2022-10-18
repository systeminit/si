<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <StatusBarTabPanelComponentList
      :component-list="filteredComponentsList"
      :selected-filter="selectedFilter"
      :filter-options="filterOptions"
      @filter="changeSelectedFilter"
    >
      <template #icon="{ component }">
        <StatusIndicatorIcon
          type="qualification"
          :status="qualificationStatusByComponentId[component.id]"
        />
      </template>
    </StatusBarTabPanelComponentList>

    <!-- Selected component view -->
    <QualificationViewerMultiple
      v-if="selectedComponent"
      :component-id="selectedComponent.id"
    />
    <div
      v-else
      class="flex flex-row items-center text-center flex-grow h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import _ from "lodash";
import QualificationViewerMultiple from "@/organisms/StatusBarTabs/Qualification/QualificationViewerMultiple.vue";
import StatusBarTabPanelComponentList, {
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { useComponentsStore } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";

const qualificationsStore = useQualificationsStore();
const componentsStore = useComponentsStore();
const selectedComponent = computed(() => componentsStore.selectedComponent);

const qualificationStatusByComponentId = computed(
  () => qualificationsStore.qualificationStatusByComponentId,
);

const defaultFilterOption = {
  value: "all",
  title: "Show All",
};
const filterOptions: FilterOption[] = [
  defaultFilterOption,
  {
    value: "success",
    title: "Success",
  },
  {
    value: "failure",
    title: "Failure",
  },
];

const selectedFilter = ref<FilterOption>(defaultFilterOption);
const changeSelectedFilter = (newFilter: FilterOption) => {
  selectedFilter.value = newFilter;
};

const componentsList = computed(() => {
  return _.map(componentsStore.allComponents, (c) => ({
    id: c.id,
    name: c.displayName,
    status: qualificationsStore.qualificationStatusByComponentId[c.id],
  }));
});
const filteredComponentsList = computed(() => {
  if (selectedFilter.value.value === "all") return componentsList.value;
  if (selectedFilter.value.value === "success")
    return _.filter(componentsList.value, { status: "success" });
  if (selectedFilter.value.value === "failure")
    return _.filter(componentsList.value, { status: "failure" });
  return [];
});
</script>
