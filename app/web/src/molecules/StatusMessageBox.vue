<template>
  <div :class="divClasses" class="flex p-2 border rounded items-start">
    <StatusIndicatorIcon
      v-if="status"
      type="confirmation"
      :status="status"
      class="w-8 mr-2 shrink-0"
    />

    <span class="self-center">
      <slot></slot>
    </span>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import StatusIndicatorIcon, {
  Status,
} from "@/molecules/StatusIndicatorIcon.vue";

const props = defineProps({
  status: { type: String as PropType<Status> },
});

const divClasses = computed(() => {
  switch (true) {
    case props.status === "success":
      return "border-success-600 text-success-500";
    case props.status === "failure":
      return "border-destructive-600 text-destructive-500";
    case props.status === "running":
      return "border-action-600 text-action-500";
    default:
      return "";
  }
});
</script>
