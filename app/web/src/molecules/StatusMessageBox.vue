<template>
  <div :class="divClasses" class="flex p-2 border rounded items-start">
    <StatusIndicatorIcon
      v-if="status"
      :status="status"
      class="w-8 mr-2 shrink-0"
    />
    <HealthIcon v-if="health" :health="health" />

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
import { ResourceHealth } from "@/api/sdf/dal/resource";
import HealthIcon from "./HealthIcon.vue";

const props = defineProps<{
  status?: Status;
  health?: ResourceHealth;
}>();

const divClasses = computed(() => {
  switch (true) {
    case props.status === "success" || props.health === ResourceHealth.Ok:
      return "border-success-600 text-success-500";
    case props.health === ResourceHealth.Warning:
      return "border-warning-600 text-warning-500";
    case props.status === "failure" || props.health === ResourceHealth.Error:
      return "border-destructive-600 text-destructive-500";
    case props.status === "loading":
      return "border-action-600 text-action-500";
    default:
      return "";
  }
});
</script>
