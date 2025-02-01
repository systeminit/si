<template>
  <div
    ref="divRef"
    v-tooltip="tooltip"
    :class="
      clsx(
        !expanded && 'truncate',
        expandOnClick && tooltip.content && 'cursor-pointer',
      )
    "
    @click="toggleExpand"
  >
    <template v-if="expandableStringArray">
      <template v-if="expanded">
        <div v-for="s in expandableStringArray" :key="s">{{ s }}</div>
      </template>
      <template v-else>{{ expandableStringArray.join(", ") }}</template>
    </template>
    <slot v-else />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { ref, computed } from "vue";

const props = defineProps({
  hasParentTruncateWithTooltip: { type: Boolean },
  showTooltip: { type: Boolean },
  expandOnClick: { type: Boolean },
  expandableStringArray: { type: Array<string> },
});

const expanded = ref(false);
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
      if (props.expandableStringArray) {
        return {
          content: expanded.value ? "Click to collapse" : "Click to expand",
        };
      }
      return {
        content: divRef.value.innerText,
      };
    }
  }
  return {};
});

const toggleExpand = () => {
  if (!props.expandOnClick || !tooltip.value.content) return;
  else expanded.value = !expanded.value;
};

defineExpose({ tooltipActive });
</script>
