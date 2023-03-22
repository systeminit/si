<template>
  <div
    v-if="computedMessage || $slots.default || noMessage"
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
import { ApiRequestStatus } from "../../pinia";
import { Icon } from "..";

const props = defineProps({
  noMessage: { type: Boolean, default: false },
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
