<template>
  <div class="flex flex-row h-full w-full">
    <StatusBarTabPanelComponentList
      :component-list="componentsList"
      :selected-filter="selectedFilter"
      :filter-options="filterOptions"
      @filter="changeSelectedFilter"
    />
    <ConfirmationsResourceList
      v-if="selectedComponent !== undefined"
      :component="selectedComponent"
      :resources="resourcesList"
      :selected="selectedResourceId"
      @select="selectResource"
    />
    <div
      v-if="selectedComponent === undefined || selectedResourceId === undefined"
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">
        {{
          selectedComponent === undefined
            ? "No Component Selected"
            : "No Resource Selected"
        }}
      </p>
    </div>
    <ConfirmationViewerMultiple
      v-else-if="selectedResource"
      :resource="selectedResource"
    />
    <div
      v-else
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">ERROR</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import StatusBarTabPanelComponentList, {
  ComponentListItem,
  FilterOption,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { SelectionService } from "@/service/selection";
import { ResourceService, fakeResources } from "@/service/resource";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";
import ConfirmationViewerMultiple from "./ConfirmationViewerMultiple.vue";

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

const confirmationsSummary = ResourceService.useResourceSummary();

const selectedComponentId = SelectionService.useSelectedComponentId();

const selectedResourceId = ref(undefined as undefined | number);

const selectResource = (id: number) => {
  selectedResourceId.value = id;
};

const resourcesList = computed(() => {
  if (selectedComponent.value) return fakeResources(selectedComponent.value);
  else return [];
});

watch(selectedComponentId, () => {
  selectedResourceId.value = undefined;
});

const componentsList = computed((): ComponentListItem[] => {
  if (confirmationsSummary.value === undefined) return [];
  const list: ComponentListItem[] = [];
  for (const component of confirmationsSummary.value.components) {
    list.push({
      id: component.id,
      name: component.name,
      type: component.type,
      health: component.health,
    });
  }
  return list;
});

const selectedComponent = computed(() => {
  return componentsList.value.find((c) => {
    return c.id === selectedComponentId.value;
  });
});

const selectedResource = computed(() => {
  return resourcesList.value.find((r) => {
    return r.id === selectedResourceId.value;
  });
});
</script>
