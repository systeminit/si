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
    <!-- fix overflow on things like super long URLs see https://github.com/tailwindlabs/tailwindcss/discussions/2213 -->
    <div class="flex-grow" :style="{ overflowWrap: 'anywhere' }">
      <slot>{{ computedMessage }}</slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import clsx from "clsx";
import { ApiRequestStatus } from "../../pinia";
import { Tones } from "../utils/color_utils";
import { Icon } from "..";

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
