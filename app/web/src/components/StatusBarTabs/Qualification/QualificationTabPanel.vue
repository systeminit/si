<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <StatusBarTabPanelComponentList
      :componentList="filteredComponentsList"
      :selectedFilter="selectedFilter"
      :filterOptions="filterOptions"
      @filter="changeSelectedFilter"
    >
      <template #icon="{ component }">
        <Icon
          v-if="component.changeStatus === 'deleted'"
          name="x"
          size="xl"
          class="text-destructive-500"
        />
        <StatusIndicatorIcon
          v-else
          type="qualification"
          :status="qualificationStatusByComponentId[component.id]"
        />
      </template>
    </StatusBarTabPanelComponentList>

    <!-- Selected component view -->
    <div
      v-if="selectedComponent?.changeStatus === 'deleted'"
      class="bg-shade-100 h-full w-full flex flex-row items-center"
    >
      <div class="text-2xl text-center w-full font-bold text-neutral-500">
        The selected component has been deleted in this change set.
      </div>
    </div>
    <QualificationViewerMultiple
      v-if="selectedComponent"
      :componentId="selectedComponent.id"
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
import * as _ from "lodash-es";
import { Icon } from "@si/vue-lib/design-system";
import QualificationViewerMultiple from "@/components/StatusBarTabs/Qualification/QualificationViewerMultiple.vue";
import StatusBarTabPanelComponentList, {
  FilterOption,
} from "@/components/StatusBar/StatusBarTabPanelComponentList.vue";
import { useComponentsStore } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";

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
    value: "warning",
    title: "Warning",
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
    changeStatus: c.changeStatus,
    status: qualificationsStore.qualificationStatusByComponentId[c.id],
  }));
});
const filteredComponentsList = computed(() => {
  const componentsListWithoutDeleted = _.filter(componentsList.value, (c) => {
    return c.changeStatus !== "deleted";
  });

  if (selectedFilter.value.value === "all") return componentsListWithoutDeleted;
  if (selectedFilter.value.value === "success")
    return _.filter(componentsListWithoutDeleted, { status: "success" });
  if (selectedFilter.value.value === "warning")
    return _.filter(componentsListWithoutDeleted, { status: "warning" });
  if (selectedFilter.value.value === "failure")
    return _.filter(componentsListWithoutDeleted, { status: "failure" });
  return [];
});
</script>
