<template>
  <div
    v-if="status === 'Ok'"
    class="flex flex-row whitespace-nowrap items-center text-sm"
  >
    <Icon
      name="check-circle"
      class="text-success-500"
      :class="!removeRightPadding ? 'pr-2' : ''"
      :size="size"
    />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
  <div
    v-else-if="status === 'Warning'"
    class="flex flex-row whitespace-nowrap items-center text-sm"
  >
    <Icon
      name="exclamation-circle"
      class="text-warning-300"
      :class="!removeRightPadding ? 'pr-2' : ''"
      :size="size"
    />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
  <div
    v-else-if="status === 'Error'"
    class="flex flex-row whitespace-nowrap items-center text-sm"
  >
    <Icon
      name="x-circle"
      class="text-destructive-500"
      :class="!removeRightPadding ? 'pr-2' : ''"
      :size="size"
    />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
  <div v-else class="flex flex-row whitespace-nowrap items-center text-sm">
    <Icon
      name="help-circle"
      class="text-neutral-300"
      :class="!removeRightPadding ? 'pr-2' : ''"
      :size="size"
    />
    <span v-if="!hideText">Health: {{ status }}</span>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import Icon, { IconSizes } from "@/ui-lib/Icon.vue";

export type WorkflowStatus = "running" | "success" | "failure";

export type Health = "Ok" | "Warning" | "Error" | "Unknown";

defineProps({
  status: {
    type: String as PropType<Health>,
    required: true,
  },
  hideText: { type: Boolean, default: false },
  size: {
    type: String as PropType<IconSizes>,
    default: "lg",
  },
  removeRightPadding: { type: Boolean, default: false },
});
</script>
