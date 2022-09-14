<template>
  <div
    v-if="status === 'Ok'"
    class="flex flex-row whitespace-nowrap items-center text-sm"
  >
    <Icon name="check-circle" class="pr-2 text-success-500" :size="size" />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
  <div
    v-else-if="status === 'Warning'"
    class="flex flex-row whitespace-nowrap items-center text-sm"
  >
    <Icon
      name="exclamation-circle"
      class="pr-2 text-warning-300"
      :size="size"
    />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
  <div
    v-else-if="status === 'Error'"
    class="flex flex-row whitespace-nowrap items-center text-sm"
  >
    <Icon name="x-circle" class="pr-2 text-destructive-500" :size="size" />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
  <div v-else class="flex flex-row whitespace-nowrap items-center text-sm">
    <Icon name="help-circle" class="pr-2 text-neutral-300" :size="size" />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import Icon, { IconSizes } from "@/ui-lib/Icon.vue";

export type WorkflowStatus = "running" | "success" | "failure";

export type HealthStatus = "Ok" | "Warning" | "Error" | "Unknown";

defineProps({
  status: {
    type: String as PropType<HealthStatus>,
    required: true,
  },
  hideText: { type: Boolean, default: false },
  size: {
    type: String as PropType<IconSizes>,
    default: "lg",
  },
});
</script>
