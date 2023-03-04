<template>
  <div>
    <LoadingMessage
      :request-status="requestStatus"
      :request-message="loadingMessage"
      :no-message="showLoaderWithoutMessage"
    />
    <ErrorMessage :request-status="requestStatus" />
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { ApiRequestStatus } from "@si/vue-lib";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import LoadingMessage from "@/ui-lib/LoadingMessage.vue";

defineProps({
  showLoaderWithoutMessage: { type: Boolean, default: false },
  loadingMessage: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});
</script>
