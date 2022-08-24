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
    <QualificationViewerMultiple
      v-if="selectedQualificationSummaryForComponent"
      :component-id="selectedQualificationSummaryForComponent.componentId"
      :component-name="selectedQualificationSummaryForComponent.componentName"
      :component-qualification-status="
        iconStatus(selectedQualificationSummaryForComponent)
      "
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
import { QualificationSummaryForComponent } from "@/service/qualification/get_summary";
import { QualificationService } from "@/service/qualification";
import { computed, ref } from "vue";
import QualificationViewerMultiple from "@/organisms/StatusBarTabs/Qualification/QualificationViewerMultiple.vue";
import { Status } from "@/molecules/StatusIndicatorIcon.vue";
import StatusBarTabPanelComponentList, {
  ComponentListItem,
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { SelectionService } from "@/service/selection";

// Loads data for qualifications - total, succeeded, failed
const qualificationSummary = QualificationService.useQualificationSummary();

const selectedComponentId = SelectionService.useSelectedComponentId();
const selectedQualificationSummaryForComponent = computed(
  (): QualificationSummaryForComponent | null => {
    if (qualificationSummary.value) {
      for (const component of qualificationSummary.value.components) {
        if (component.componentId === selectedComponentId.value) {
          return component;
        }
      }
    }
    return null;
  },
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

const list = computed((): ComponentListItem[] => {
  if (qualificationSummary.value === undefined) return [];
  let components = qualificationSummary.value.components;
  if (selectedFilter.value.value === "success")
    components = components.filter((component) => component.failed === 0);
  else if (selectedFilter.value.value === "failure")
    components = components.filter((component) => component.failed > 0);

  const list: ComponentListItem[] = [];
  for (const component of components) {
    list.push({
      id: component.componentId,
      name: component.componentName,
      status: iconStatus(component),
    });
  }
  return list;
});

const iconStatus = (component: QualificationSummaryForComponent): Status =>
  component.succeeded === component.total
    ? "success"
    : component.failed + component.succeeded === component.total
    ? "failure"
    : "loading";
</script>
