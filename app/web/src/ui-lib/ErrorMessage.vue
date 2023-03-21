<template>
  <div
    v-if="computedMessage || $slots.default"
    :class="
      clsx(
        'border p-xs text-sm rounded-sm flex flex-row items-center',
        tone === 'destructive' && 'border-destructive-500 text-destructive-400',
        tone === 'warning' && 'border-warning-500 text-warning-400',
      )
    "
  >
    <Icon name="alert-triangle" class="mr-xs flex-none" />
    <div class="flex-grow">
      <slot>{{ computedMessage }}</slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import clsx from "clsx";
import { ApiRequestStatus } from "@si/vue-lib";
import Icon from "./icons/Icon.vue";
import { Tones } from "./helpers/tones";

const props = defineProps({
  message: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
  tone: { type: String as PropType<Tones>, default: "destructive" },
});

const computedMessage = computed(() => {
  if (props.message) return props.message;
  return props.requestStatus?.errorMessage;
});
</script>
