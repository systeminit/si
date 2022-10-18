<template>
  <div class="flex flex-row h-full w-full">
    <ConfirmationsResourceList />
    <div
      v-if="!selectedResource"
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Resource Selected</p>
    </div>
    <ConfirmationViewerMultiple v-else />
    <!-- <div
      v-else
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">ERROR</p>
    </div> -->
  </div>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { computed } from "vue";
import { useResourcesStore } from "@/store/resources.store";

import ConfirmationViewerMultiple from "./ConfirmationViewerMultiple.vue";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";

const resourcesStore = useResourcesStore();

const selectedResource = computed(() => resourcesStore.selectedResource);
</script>
