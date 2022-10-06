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
    <ConfirmationViewerMultiple v-else-if="resource" :resource="resource" />
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
import { ResourceService } from "@/service/resource";
import { useComponentsStore } from "@/store/components.store";
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

const componentsStore = useComponentsStore();
const selectedComponentId = computed(() => componentsStore.selectedComponentId);

const selectedResourceId = ref(undefined as undefined | number);

const selectResource = (id: number) => {
  selectedResourceId.value = id;
};

const resourceSummary = ResourceService.useResourceSummary();

const resource = computed(() => {
  console.log(selectedComponent.value?.resource);
  if (selectedComponent.value && selectedComponent.value.resource) {
    return selectedComponent.value.resource;
  }
  return undefined;
});

// FIXME: since there can only be one (or none) Resource for a Component and System, this list
// should be replaced by a single Resource. For now, this is used to maintain compatability.
const resourcesList = computed(() => {
  const resources = [];
  if (resource.value) {
    resources.push(resource.value);
  }
  return resources;
});

watch(selectedComponentId, () => {
  selectedResourceId.value = undefined;
});

const componentsList = computed((): ComponentListItem[] => {
  if (resourceSummary.value === undefined) return [];
  const list: ComponentListItem[] = [];
  for (const component of resourceSummary.value.components) {
    list.push({
      id: component.id,
      name: component.name,
      schema: component.schema,
      health: component.health,
      resource: component.resource,
    });
  }
  return list;
});

const selectedComponent = computed(() => {
  return componentsList.value.find((c) => {
    return c.id === selectedComponentId.value;
  });
});
</script>
