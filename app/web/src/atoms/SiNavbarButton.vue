<template>
  <div :class="bgClasses">
    <button
      v-tooltip.bottom="tooltipText"
      :class="buttonClasses"
      :aria-label="props.tooltipText"
      :disabled="disabled"
      @click="emit('click')"
    >
      <slot></slot>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText: string;
  panelSwitcher?: boolean;
}>();
const { disabled, selected, tooltipText } = toRefs(props);

const selectedPanelBgColor = "bg-[#2F80ED]";
const selectedBgColor = "bg-black";

const bgClasses = computed(() => {
  const results: Record<string, boolean> = {
    "py-12": true,
    "px-4": true,
    "hover:bg-black": true,
  };

  if (selected?.value) {
    results["hover:bg-black"] = false;
    if (props.panelSwitcher) {
      results[selectedPanelBgColor] = true;
    } else {
      results[selectedBgColor] = true;
    }
  }

  return results;
});

const buttonClasses = computed(() => {
  const results: Record<string, boolean> = {
    block: true,
    "w-6": true,
    "h-6": true,
    "text-gray-300": true,
    "hover:text-white": true,
  };
  if (disabled?.value) {
    results["opacity-50"] = true;
    results["cursor-not-allowed"] = true;
  } else if (selected?.value) {
    results["text-white"] = true;
    results["text-gray-300"] = false;
    results["hover:text-white"] = false;
  }
  return results;
});
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
