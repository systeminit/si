<template>
  <div>
    <slot v-if="status === 'loading'" :message="loadingMessage">
      <LoadingMessage
        :message="loadingMessage"
        :requestStatus="requestStatus"
        :asyncState="asyncState"
        :noPadding="noPadding"
      />
    </slot>
    <slot v-if="status === 'error'" name="error" :message="errorMessage">
      <ErrorMessage
        :requestStatus="requestStatus"
        :asyncState="asyncState"
        :noPadding="noPadding"
      />
    </slot>
    <slot v-if="status === 'success'" name="success" :value="value" />
    <slot v-if="status === 'uninitialized'" name="uninitialized" />
  </div>
</template>

<script lang="ts" setup>
import { UseAsyncStateReturn } from "@vueuse/core";
import { computed } from "vue";
import { ApiRequestStatus, getErrorMessage, getLoadStatus } from "../../pinia";
import ErrorMessage from "./ErrorMessage.vue";
import LoadingMessage from "./LoadingMessage.vue";

const props = defineProps<{
  requestStatus?: ApiRequestStatus;
  asyncState?: UseAsyncStateReturn<unknown, unknown[], boolean>;
  loadingMessage?: string;
  noPadding?: boolean;
}>();

const errorMessage = computed(() => getErrorMessage(props));
const status = computed(() => getLoadStatus(props));
const value = computed(() => props.asyncState?.state.value);
</script>
