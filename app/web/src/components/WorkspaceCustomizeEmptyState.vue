<template>
  <div class="w-full h-full flex flex-col items-center gap-sm py-lg overflow-hidden">
    <div class="max-w-96">
      <EmptyStateIcon name="customize-big" />
    </div>
    <div class="text-lg font-medium text-center text-neutral-600 dark:text-neutral-300">
      Customize ASSETS, FUNCTIONS, and MODULES
    </div>
    <div class="text-sm italic text-neutral-500 dark:text-neutral-400 text-center">
      {{ $props.instructions }}
    </div>
    <div>
      <RequestStatusMessage
        v-if="!requestStatus.isSuccess"
        :requestStatus="requestStatus"
        :loadingMessage="loadingMessage"
      />
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
  instructions: {
    type: String,
    required: false,
    default() {
      return "Use the left panel to get started!";
    },
  },
});
</script>
