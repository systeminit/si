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
import ErrorMessage from "./ErrorMessage.vue";
import LoadingMessage from "./LoadingMessage.vue";
import { ApiRequestStatus } from "../../pinia";

defineProps<{
  loadingMessage?: string;
  requestStatus?: ApiRequestStatus;
  noPadding?: boolean;
}>();
</script>
