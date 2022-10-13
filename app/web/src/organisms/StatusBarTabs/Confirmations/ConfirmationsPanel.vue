<template>
  <div class="flex flex-row h-full w-full">
    <ConfirmationsResourceList
      :resources="resourcesList"
      :selected="selectedResourceId"
      @select="selectResource"
    />
    <div
      v-if="selectedResourceId === undefined"
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Resource Selected</p>
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
import { ComponentListItem } from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import {
  ResourceService,
  MockResource,
  Confirmation,
} from "@/service/resource";
import { useFixesStore } from "@/store/fixes/fixes.store";
import { ResourceHealth, ResourceStatus } from "@/api/sdf/dal/resource";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";
import ConfirmationViewerMultiple from "./ConfirmationViewerMultiple.vue";

const selectedResourceId = ref(undefined as undefined | number);

const selectResource = (id: number) => {
  selectedResourceId.value = id;
};

const resourceSummary = ResourceService.useResourceSummary();

const resourcesList = computed((): MockResource[] => {
  if (resourceSummary.value === undefined) {
    const empty: MockResource[] = [];
    return empty;
  }
  const list: MockResource[] = [];
  for (const component of resourceSummary.value.components) {
    const fix = fixes.value[component.id];
    let created = false;
    if (fix) {
      const fixStatus = fix.status;
      if (fixStatus === "success") created = true;
    }
    const confirmations: Confirmation[] = [
      created
        ? {
            title: "Does The Resource Exist?",
            health: "ok" as ResourceHealth,
            description:
              "Checks if the resource actually exists. This resource exists!",
          }
        : {
            title: "Does The Resource Exist?",
            health: "error" as ResourceHealth,
            description:
              "Checks if the resource actually exists. This resource has not been created yet. Please run the fix above to create it!",
          },
    ];
    const resource: MockResource = {
      id: component.id,
      name: component.name,
      kind: component.schema,
      health: created ? ("ok" as ResourceHealth) : ("error" as ResourceHealth),
      status: created
        ? ("Created" as ResourceStatus)
        : ("Pending" as ResourceStatus),
      confirmations,
    };
    list.push(resource);
  }
  return list;
});

const selectedResource = computed(() => {
  return resourcesList.value.find((r) => {
    return r.id === selectedResourceId.value;
  });
});

const fixesStore = useFixesStore();
const fixes = computed(() => fixesStore.$state.fixesById);
</script>
