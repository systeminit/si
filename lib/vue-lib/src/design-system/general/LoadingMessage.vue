<template>
  <div
    v-if="
      requestStatus?.isPending ||
      asyncState?.isLoading ||
      (!requestStatus && !asyncState)
    "
    :class="
      clsx('w-full flex flex-col items-center gap-sm', !noPadding && 'p-xl')
    "
  >
    <Icon name="loader" size="2xl" />
    <h2 v-if="message || $slots.default" class="text-lg">
      <slot>{{ message }}</slot>
    </h2>
    <div v-if="$slots.moreContent">
      <slot name="moreContent" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { UseAsyncStateReturn } from "@vueuse/core";
import clsx from "clsx";
import { ApiRequestStatus } from "../../pinia";
import { Icon } from "..";

// Convert this to the defineProps() syntax with PropType
defineProps<{
  message?: string;
  requestStatus?: ApiRequestStatus;
  asyncState?: UseAsyncStateReturn<unknown, unknown[], boolean>;
  noPadding?: boolean;
}>();
</script>
