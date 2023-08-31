<template>
  <div
    v-if="requestStatus === undefined || requestStatus.isPending"
    class="w-full flex flex-col items-center gap-4 p-xl"
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
import { PropType } from "vue";
import { ApiRequestStatus } from "../../pinia";
import { Icon } from "..";

const props = defineProps({
  message: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});
</script>
