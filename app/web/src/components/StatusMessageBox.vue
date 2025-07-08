<template>
  <div :class="divClasses" class="flex p-xs border rounded items-start">
    <StatusIndicatorIcon
      v-if="status"
      :type="type"
      :status="status"
      class="w-8 mr-2 shrink-0"
    />

    <span class="self-center grow">
      <slot></slot>
    </span>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import StatusIndicatorIcon, {
  IconType,
  Status,
} from "@/components/StatusIndicatorIcon.vue";

const props = defineProps({
  status: { type: String as PropType<Status> },
  type: { type: String as PropType<IconType>, default: "qualification" },
});

const divClasses = computed(() => {
  switch (true) {
    case props.status === "success":
      return "border-success-600 text-success-500";
    case props.status === "warning":
      return "border-warning-600 text-warning-500";
    case props.status === "failure":
      return "border-destructive-600 text-destructive-600";
    case props.status === "running":
      return "border-action-600 text-action-500";
    default:
      return "";
  }
});
</script>
