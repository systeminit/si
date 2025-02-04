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
        <TruncateWithTooltip v-for="s in expandableStringArray" :key="s">{{
          s
        }}</TruncateWithTooltip>
      </template>
      <template v-else>{{ expandableStringArray.join(", ") }}</template>
    </template>
    <slot v-else />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { ref, computed, onMounted } from "vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue"; // eslint-disable-line import/no-self-import

const props = defineProps({
  hasParentTruncateWithTooltip: { type: Boolean },
  showTooltip: { type: Boolean },
  expandOnClick: { type: Boolean },
  expandableStringArray: { type: Array<string> },
});

const expanded = ref(false);
const divRef = ref<HTMLElement>();

const neededTooltipOnLoad = ref(false);

onMounted(() => {
  if (divRef.value) {
    neededTooltipOnLoad.value =
      divRef.value.clientWidth < divRef.value.scrollWidth;
  }
});

const tooltipActive = computed(() => {
  if (divRef.value && divRef.value.clientWidth < divRef.value.scrollWidth) {
    return true;
  }
  return false;
});

const tooltip = computed(() => {
  if (neededTooltipOnLoad.value && props.expandableStringArray) {
    return {
      content: expanded.value ? "Click to collapse" : "Click to expand",
    };
  } else if (
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

const toggleExpand = () => {
  if (!props.expandOnClick || !tooltip.value.content) return;
  else expanded.value = !expanded.value;
};

defineExpose({ tooltipActive });
</script>
