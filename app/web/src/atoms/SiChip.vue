<template>
  <div
    class="inline-block border-solid border rounded py-1 px-1"
    :class="colorClasses"
  >
    {{ text }}
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";

const props = defineProps<{
  text: string;
  variant?: "warning" | "neutral";
}>();

const colorClasses = computed(() => {
  switch (props.variant) {
    case "warning":
      return "border-warning-700 text-warning-700 bg-warning-100";
    default: // neutral is default
      return "border-action-700 text-action-700 bg-action-100";
  }
});
</script>
