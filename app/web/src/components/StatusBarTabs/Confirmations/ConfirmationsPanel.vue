<template>
  <div class="flex flex-row h-full w-full">
    <ConfirmationsResourceList />
    <div
      v-if="
        !componentsStore.selectedComponentId ||
        !componentIds.includes(componentsStore.selectedComponentId)
      "
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">
        {{
          !componentsStore.selectedComponentId
            ? "No Component Selected"
            : "Selected Component Does Not Have A Resource"
        }}
      </p>
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
import { useFixesStore } from "@/store/fixes.store";
import ConfirmationViewerMultiple from "./ConfirmationViewerMultiple.vue";
import ConfirmationsResourceList from "./ConfirmationsResourceList.vue";

const fixesStore = useFixesStore();
const componentIds = computed(() => fixesStore.allComponents);
const componentsStore = useComponentsStore();
</script>
