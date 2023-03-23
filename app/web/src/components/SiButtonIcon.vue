<template>
  <button
    v-tooltip.bottom="tooltipText"
    :aria-label="props.tooltipText"
    :class="buttonClasses"
    :disabled="disabled"
    @click="emit('click')"
  >
    <Icon :name="icon" />
  </button>
</template>

<script lang="ts" setup>
// TODO(victor) this component can probably be replaced by our VButton with showLabel=false
import { computed, toRefs } from "vue";
import { Icon, IconNames } from "@si/vue-lib/design-system";

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText?: string;
  ignoreTextColor?: boolean;
  icon: IconNames;
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
      results["text-action-300"] = true;
      results["hover:text-action-200"] = true;
    } else if (!props.ignoreTextColor) {
      results["text-neutral-300"] = true;
      results["hover:text-neutral-100"] = true;
    }
  }
  return results;
});
</script>

<style scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
