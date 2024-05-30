<template>
  <div>
    <LoadingMessage
      :requestStatus="requestStatus"
      :message="loadingMessage"
      :noPadding="noPadding"
    />
    <ErrorMessage :requestStatus="requestStatus" :noPadding="noPadding" />
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import ErrorMessage from "./ErrorMessage.vue";
import LoadingMessage from "./LoadingMessage.vue";
import { ApiRequestStatus } from "../../pinia";

defineProps({
  loadingMessage: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
  noPadding: { type: Boolean },
});
</script>
