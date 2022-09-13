<template>
  <div
    v-if="status === 'success'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <Icon name="check-circle" class="pr-2 text-success-500" :size="size" />
    <span v-if="showText">Success!</span>
  </div>
  <div
    v-else-if="status === 'failure'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <Icon name="x-circle" class="pr-2 text-destructive-500" :size="size" />
    <span v-if="showText">Failure</span>
  </div>
  <div v-else class="flex flex-row whitespace-nowrap items-center">
    <Icon name="loader" class="pr-2 text-action-300" :size="size" />
    <span v-if="showText">Running...</span>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import Icon, { IconSizes } from "@/ui-lib/Icon.vue";

export type WorkflowStatus = "running" | "success" | "failure";

defineProps({
  status: {
    type: String as PropType<"running" | "success" | "failure">,
    required: true,
  },
  showText: { type: Boolean, default: false },
  size: {
    type: String as PropType<IconSizes>,
    default: "xl",
  },
});
</script>
