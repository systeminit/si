<template>
  <div class="flex flex-row h-full w-full">
    <ConfirmationsResourceList />
    <div
      v-if="!hasSelectedResource"
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
import * as _ from "lodash-es";
import { computed } from "vue";
import { useComponentsStore } from "@/store/components.store";

import ConfirmationViewerMultiple from "./ConfirmationViewerMultiple.vue";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";

const componentsStore = useComponentsStore();
const hasSelectedResource = computed(
  () => !!componentsStore.selectedComponentId,
);
</script>
