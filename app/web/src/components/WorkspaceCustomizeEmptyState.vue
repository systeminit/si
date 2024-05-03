<template>
  <div
    v-if="requestStatus.isPending"
    class="text-center text-neutral-400 dark:text-neutral-300 w-full h-full"
  >
    <RequestStatusMessage
      :requestStatus="requestStatus"
      :loadingMessage="loadingMessage"
    />
  </div>
  <div
    v-else-if="requestStatus.isSuccess"
    class="w-full h-full flex flex-col items-center gap-sm py-lg"
  >
    <div class="max-w-96">
      <EmptyStateIcon name="customize-big" />
    </div>
    <div
      class="text-lg font-medium text-center text-neutral-600 dark:text-neutral-300"
    >
      Customize ASSETS, FUNCTIONS, and MODULES
    </div>
    <div
      class="text-sm italic text-neutral-500 dark:text-neutral-400 text-center"
    >
      Use the left panel to get started!
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import { PropType } from "vue";
import { RequestStatusMessage } from "@si/vue-lib/design-system";
import EmptyStateIcon from "./EmptyStateIcon.vue";

defineProps({
  requestStatus: { type: Object as PropType<ApiRequestStatus>, required: true },
  loadingMessage: { type: String, required: true },
});
</script>
