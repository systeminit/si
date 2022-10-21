<template>
  <div
    v-if="computedMessage || $slots.default"
    class="border border-destructive-500 text-destructive-400 p-xs text-sm rounded-sm flex flex-row items-center"
  >
    <Icon name="alert-triangle" class="mr-xs flex-none" />
    <div class="flex-grow">
      <slot>{{ computedMessage }}</slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import { ApiRequestStatus } from "@/utils/pinia_api_tools";
import Icon from "./icons/Icon.vue";

const props = defineProps({
  message: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const computedMessage = computed(() => {
  if (props.message) return props.message;
  return props.requestStatus?.errorMessage;
});
</script>
