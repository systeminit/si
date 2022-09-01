<template>
  <div
    v-if="status === 'success'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <div class="pr-2 text-success-500"><Icon name="check-circle" /></div>
    <span v-if="showText">Success!</span>
  </div>
  <div
    v-else-if="status === 'failure'"
    class="flex flex-row whitespace-nowrap items-center"
  >
    <div class="pr-2 text-destructive-500"><Icon name="x-circle" /></div>
    <span v-if="showText">Failure</span>
  </div>
  <div v-else class="flex flex-row whitespace-nowrap items-center">
    <div class="pr-2"><Icon name="loader" /></div>
    <span v-if="showText">Running...</span>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import Icon from "@/ui-lib/Icon.vue";

export type WorkflowStatus = "running" | "success" | "failure";

defineProps({
  status: {
    type: String as PropType<"running" | "success" | "failure">,
    required: true,
  },
  showText: { type: Boolean, default: false },
});
</script>
