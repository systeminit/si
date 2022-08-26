<template>
  <div :class="divClasses" class="flex p-2 border rounded items-start">
    <StatusIndicatorIcon :status="status" class="w-8 mr-2 shrink-0" />

    <span class="self-center">
      <slot></slot>
    </span>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import StatusIndicatorIcon, {
  Status,
} from "@/molecules/StatusIndicatorIcon.vue";

const props = defineProps<{
  status: Status;
}>();

const divClasses = computed(() => {
  switch (props.status) {
    case "success":
      return "border-success-600 text-success-500";
    case "failure":
      return "border-destructive-600 text-destructive-500";
    case "loading":
      return "border-action-600 text-action-500";
    default:
      return "";
  }
});
</script>
