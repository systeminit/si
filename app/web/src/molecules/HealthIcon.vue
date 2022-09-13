<template>
  <div
    v-if="status === 'Ok'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <Icon name="check-circle" class="pr-2 text-success-500" :size="size" />
    <span v-if="showText">Health: {{ status }}</span>
  </div>
  <div
    v-else-if="status === 'Warning'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <Icon
      name="exclamation-circle"
      class="pr-2 text-warning-300"
      :size="size"
    />
    <span v-if="showText">Health: {{ status }}</span>
  </div>
  <div
    v-else-if="status === 'Error'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <Icon name="x-circle" class="pr-2 text-destructive-500" :size="size" />
    <span v-if="showText">Health: {{ status }}</span>
  </div>
  <div v-else class="flex flex-row whitespace-nowrap items-center">
    <Icon name="help-circle" class="pr-2 text-neutral-300" :size="size" />
    <span v-if="showText">Health: {{ status }}</span>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import Icon, { IconSizes } from "@/ui-lib/Icon.vue";

export type WorkflowStatus = "running" | "success" | "failure";

defineProps({
  status: {
    type: String as PropType<"Ok" | "Warning" | "Error" | "Unknown">,
    required: true,
  },
  showText: { type: Boolean, default: true },
  size: {
    type: String as PropType<IconSizes>,
    default: "xl",
  },
});
</script>
