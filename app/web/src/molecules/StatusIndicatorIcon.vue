<template>
  <CheckCircleIcon
    v-if="status === 'success'"
    :class="iconClasses"
    class="text-success-500"
  />
  <XCircleIcon
    v-else-if="status === 'failure'"
    :class="iconClasses"
    class="text-destructive-500"
  />
  <ClockIcon
    v-else-if="status === 'loading'"
    :class="iconClasses"
    class="text-warning-500"
  />
</template>

<script lang="ts" setup>
import {
  CheckCircleIcon,
  ClockIcon,
  XCircleIcon,
} from "@heroicons/vue/outline";
import { defineProps } from "vue";

export type Status = "success" | "failure" | "loading";

defineProps<{
  status: Status;
}>();

const iconClasses = "inline align-middle";
</script>
