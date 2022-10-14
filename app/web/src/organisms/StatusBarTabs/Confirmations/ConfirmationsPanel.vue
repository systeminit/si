<template>
  <div class="flex flex-row h-full w-full">
    <ConfirmationsResourceList
      :resources="resources"
      :selected="selectedResourceId"
      @select="onSelectResource"
    />
    <div
      v-if="!selectedResourceId"
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Resource Selected</p>
    </div>
    <ConfirmationViewerMultiple
      v-else-if="selectedResource"
      :selected-resource="selectedResource"
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
import _ from "lodash";
import { computed, ref } from "vue";
import { useResourcesStore } from "@/store/resources.store";

import ConfirmationViewerMultiple from "./ConfirmationViewerMultiple.vue";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";

const resourcesStore = useResourcesStore();
const resources = computed(() => resourcesStore.allResources);

const selectedResourceId = ref(undefined as undefined | number);
// we may want this to live in the store? depending on how it is used
const selectedResource = computed(() =>
  _.find(resources.value, (r) => r.id === selectedResourceId.value),
);
const onSelectResource = (id: number) => {
  selectedResourceId.value = id;
};
</script>
