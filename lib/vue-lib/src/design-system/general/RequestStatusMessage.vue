<template>
  <div>
    <LoadingMessage
      :requestStatus="requestStatus"
      :requestMessage="loadingMessage"
      :noMessage="showLoaderWithoutMessage"
    />
    <ErrorMessage :requestStatus="requestStatus" />
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import ErrorMessage from "./ErrorMessage.vue";
import LoadingMessage from "./LoadingMessage.vue";
import { ApiRequestStatus } from "../../pinia";

defineProps({
  showLoaderWithoutMessage: { type: Boolean, default: false },
  loadingMessage: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});
</script>
