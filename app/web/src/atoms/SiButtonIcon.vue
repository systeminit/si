<template>
  <Popper :hover="true" :open-delay="500" :content="props.tooltipText">
    <button
      :class="buttonStyle"
      :aria-label="props.tooltipText"
      :disabled="props.disabled"
      :style="{ color: props.color }"
      @click="emit('click')"
    >
      <slot></slot>
    </button>
  </Popper>
</template>

<script setup lang="ts">
import Popper from "vue3-popper";
import { computed } from "vue";

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  color: string;
  tooltipText: string;
}>();

const buttonStyle = computed(() => {
  const results: Record<string, boolean> = {};
  results["button-standard"] = true;
  results["text-xs"] = true;
  if (props.disabled) {
    results["opacity-50"] = true;
    results["cursor-not-allowed"] = true;
  }
  return results;
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.05;

.button-standard {
  display: flex;
  width: 20px;
}

.button-standard:hover {
  filter: brightness($button-brightness);
}

.button-standard:focus {
  outline: none;
}

.button-standard:active {
  filter: saturate(1.5) brightness($button-brightness);
}
</style>
