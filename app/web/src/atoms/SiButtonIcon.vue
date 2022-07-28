<template>
  <button
    v-tooltip.bottom="tooltipText"
    :class="buttonClasses"
    :aria-label="props.tooltipText"
    :disabled="disabled"
    @click="emit('click')"
  >
    <slot></slot>
  </button>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText?: string;
  ignoreTextColor?: boolean;
}>();
const { disabled, selected, tooltipText } = toRefs(props);

const buttonClasses = computed(() => {
  const results: Record<string, boolean> = {
    block: true,
    "w-5": true,
    "h-5": true,
  };
  if (disabled?.value) {
    results["opacity-50"] = true;
    results["cursor-not-allowed"] = true;
  } else {
    if (selected?.value) {
      results["text-blue-300"] = true;
      results["hover:text-blue-200"] = true;
    } else if (!props.ignoreTextColor) {
      results["text-gray-300"] = true;
      results["hover:text-gray-100"] = true;
    }
  }
  return results;
});
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
