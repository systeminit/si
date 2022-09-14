<template>
  <div class="flex flex-row h-full w-full">
    <StatusBarTabPanelComponentList
      :component-list="list"
      :selected-filter="selectedFilter"
      :filter-options="filterOptions"
      @filter="changeSelectedFilter"
    />
    <ConfirmationsResourceList
      component-name="COMPONENTNAME"
      :resources="fakeResources"
    />
    <div
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">
        {{ selectedComponentText }}
      </p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import StatusBarTabPanelComponentList, {
  ComponentListItem,
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { SelectionService } from "@/service/selection";
import { QualificationService } from "@/service/qualification";
import { Health } from "@/molecules/HealthIcon.vue";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";

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

const confirmationsSummary = QualificationService.useQualificationSummary();

const selectedComponentId = SelectionService.useSelectedComponentId();
const selectedComponentText = computed((): string => {
  if (selectedComponentId.value) {
    return `Selected Component ID: ${selectedComponentId.value}`;
  }
  return "No Component Selected";
});

const list = computed((): ComponentListItem[] => {
  if (confirmationsSummary.value === undefined) return [];
  const list: ComponentListItem[] = [];
  for (const component of confirmationsSummary.value.components) {
    list.push({
      id: component.componentId,
      name: component.componentName,
      health: "Unknown", // TODO(wendy) - put in an actual health summary for each component's resources here
    });
  }
  return list;
});

const fakeResources = ref([
  { id: 1, name: "docker image", health: "Ok" as Health },
  { id: 2, name: "other resource", health: "Error" as Health },
]);
</script>
