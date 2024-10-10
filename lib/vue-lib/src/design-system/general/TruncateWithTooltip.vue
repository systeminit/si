<template>
  <div ref="divRef" v-tooltip="tooltip" class="truncate">
    <slot />
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";

const props = defineProps({
  hasParentTruncateWithTooltip: { type: Boolean },
  showTooltip: { type: Boolean },
});

const divRef = ref<HTMLElement>();

const tooltipActive = computed(() => {
  if (divRef.value && divRef.value.clientWidth < divRef.value.scrollWidth) {
    return true;
  }
  return false;
});

const tooltip = computed(() => {
  if (
    divRef.value &&
    (props.showTooltip || divRef.value.clientWidth < divRef.value.scrollWidth)
  ) {
    if (!props.hasParentTruncateWithTooltip) {
      return {
        content: divRef.value.innerText,
      };
    }
  }
  return {};
});

defineExpose({ tooltipActive });
</script>
