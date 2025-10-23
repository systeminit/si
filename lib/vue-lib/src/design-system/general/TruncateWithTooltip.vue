<template>
  <div
    ref="divRef"
    v-tooltip="tooltip"
    :class="
      clsx(
        lineClamp
          ? ['break-words', numberToLineClamp(lineClamp)]
          : [
              !expanded && 'truncate',
              expandOnClick && tooltip.content && 'cursor-pointer',
            ],
      )
    "
    @click="toggleExpand"
  >
    <template v-if="expandableStringArray && !lineClamp">
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
import { ref, computed, onMounted, PropType, onBeforeUnmount } from "vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue"; // eslint-disable-line import/no-self-import
import { tw } from "../../utils/tw-utils";

type LineClamps = 2 | 3 | 4 | 5;

const numberToLineClamp = (n: LineClamps) => {
  return {
    2: tw`line-clamp-2`,
    3: tw`line-clamp-3`,
    4: tw`line-clamp-4`,
    5: tw`line-clamp-5`,
  }[n];
};

const props = defineProps({
  hasParentTruncateWithTooltip: { type: Boolean },
  showTooltip: { type: Boolean },
  expandOnClick: { type: Boolean },
  expandableStringArray: { type: Array<string> },
  lineClamp: { type: Number as PropType<LineClamps> }, // not compatible with the other features
});

const expanded = ref(false);
const divRef = ref<HTMLElement>();
const innerText = ref("");
const mObserver = ref();
const neededTooltipOnLoad = ref(false);
const forceRecompute = ref(0);

const windowResizeHandler = () => {
  forceRecompute.value++;
};

onMounted(() => {
  if (divRef.value) {
    neededTooltipOnLoad.value =
      divRef.value.clientWidth < divRef.value.scrollWidth;
  }

  mObserver.value = new MutationObserver(() => {
    innerText.value = divRef.value?.innerText || "";
    forceRecompute.value++; // Only way to ENSURE that the tooltip recomputes properly!
  });

  mObserver.value.observe(divRef.value, {
    subtree: true,
    characterData: true,
  });

  window.addEventListener("resize", windowResizeHandler);
});

onBeforeUnmount(() => {
  if (mObserver.value) {
    mObserver.value.disconnect();
  }
  window.removeEventListener("resize", windowResizeHandler);
});

const tooltipActive = computed(() => {
  if (divRef.value && divRef.value.clientWidth < divRef.value.scrollWidth) {
    return true;
  }
  return false;
});

const tooltip = computed(() => {
  // we invoke forceRecompute here to force the tooltip to recompute when we need it to
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  forceRecompute.value;
  if (props.lineClamp) {
    if (divRef.value && divRef.value.scrollHeight > divRef.value.clientHeight) {
      return {
        theme: "instant-show",
        content: innerText.value || divRef.value.innerText,
      };
    } else {
      return {};
    }
  } else if (neededTooltipOnLoad.value && props.expandableStringArray) {
    return {
      content: expanded.value ? "Click to collapse" : "Click to expand",
    };
  } else if (
    divRef.value &&
    (props.showTooltip || divRef.value.clientWidth < divRef.value.scrollWidth)
  ) {
    if (!props.hasParentTruncateWithTooltip) {
      return {
        theme: "instant-show",
        content: innerText.value || divRef.value.innerText,
      };
    }
  }
  return {};
});

const toggleExpand = () => {
  if (!props.expandOnClick || !tooltip.value.content || props.lineClamp) return;
  else expanded.value = !expanded.value;
};

// This catches various difficult bugs that happen when computing line clamp tooltips - not elegant but it works!
const lineClampRefresh = ref();
onMounted(() => {
  if (props.lineClamp) {
    lineClampRefresh.value = setInterval(() => {
      forceRecompute.value++;
    }, 1000);
  }
});
onBeforeUnmount(() => {
  clearInterval(lineClampRefresh.value);
});

defineExpose({ tooltipActive, tooltip });
</script>
