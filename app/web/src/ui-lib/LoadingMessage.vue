<template>
  <div
    v-if="computedMessage || $slots.default"
    class="w-full flex flex-col items-center gap-4 p-xl"
  >
    <Icon name="loader" size="2xl" />
    <h2>
      <slot>{{ computedMessage }}</slot>
    </h2>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import { ApiRequestStatus } from "@/utils/pinia_api_tools";
import Icon from "./icons/Icon.vue";

const props = defineProps({
  message: { type: String },
  requestMessage: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const computedMessage = computed(() => {
  if (props.message) return props.message;
  else if (props.requestStatus?.isPending) return props.requestMessage;
  else return undefined;
});
</script>
