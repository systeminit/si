<template>
  <div
    ref="panel"
    :class="
      clsx(
        'si-panel',
        'z-20 dark:text-white border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 pointer-events-auto relative',
        {
          left: 'border-r-2',
          right: 'border-l-2',
          top: 'border-b-2',
          bottom: 'border-t-2',
        }[side],
        `${side}-0`,
      )
    "
    :style="{
      ...(resizeable && {
        [isVertical ? 'height' : 'width']: `${currentSize}px`,
      }),
    }"
  >
    <SiPanelResizer
      v-if="resizeable"
      class="si-panel__resizer"
      :panel-side="side"
      @resize-start="onResizeStart"
      @resize-move="onResizeMove"
      @resize-reset="resetSize"
    />
    <div class="si-panel__inner absolute w-full h-full">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, PropType, ref } from "vue";
import _ from "lodash";
import clsx from "clsx";
import SiPanelResizer from "@/atoms/SiPanelResizer.vue";

const props = defineProps({
  rememberSizeKey: { type: String, required: true },
  side: {
    type: String as PropType<"left" | "right" | "top" | "bottom">,
    required: true,
  },
  hidden: { type: Boolean },
  classes: {
    type: String,
    default:
      "z-20 dark:text-white border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 pointer-events-auto relative",
  },
  sizeClasses: { type: String, default: "" },
  resizeable: { type: Boolean, default: true },
  minSizeRatio: { type: Number, default: 0.1 },
  minSize: { type: Number, default: 200 },
  maxSizeRatio: { type: Number, default: 0.45 },
  maxSize: { type: Number },
  defaultSize: { type: Number, default: 320 },
});

const isVertical = computed(
  () => props.side === "top" || props.side === "bottom",
);

const panel = ref();
const currentSize = ref(0);

const setSize = (newSize: number) => {
  let finalSize = newSize;

  // TODO: make sure these checks don't conflict with each other

  if (props.minSize) {
    if (finalSize < props.minSize) finalSize = props.minSize;
  }
  if (props.minSizeRatio) {
    const limit =
      (isVertical.value ? window.innerHeight : window.innerWidth) *
      props.minSizeRatio;
    if (finalSize < limit) finalSize = limit;
  }

  if (props.maxSize) {
    if (finalSize > props.maxSize) finalSize = props.maxSize;
  }
  if (props.maxSizeRatio) {
    const limit =
      (isVertical.value ? window.innerHeight : window.innerWidth) *
      props.maxSizeRatio;

    if (finalSize > limit) finalSize = limit;
  }
  currentSize.value = finalSize;
  if (finalSize === props.defaultSize) {
    window.localStorage.removeItem(localStorageKey.value);
  } else {
    window.localStorage.setItem(localStorageKey.value, `${finalSize}`);
  }
};

const localStorageKey = computed(() => `${props.rememberSizeKey}-size`);

const beginResizeValue = ref(0);
const onResizeStart = () => {
  beginResizeValue.value = currentSize.value;
};

const onResizeMove = (delta: number) => {
  const adjustedDelta =
    props.side === "right" || props.side === "bottom" ? delta : -delta;
  setSize(beginResizeValue.value + adjustedDelta);
};

const resetSize = (useDefaultSize = true) => {
  if (props.defaultSize && useDefaultSize) {
    setSize(props.defaultSize);
  }
};

const onWindowResize = () => {
  // may change the size because min/max ratio of window size may have changed
  setSize(currentSize.value);
};

const debounceForResize = _.debounce(onWindowResize, 20);
const windowResizeObserver = new ResizeObserver(debounceForResize);

onMounted(() => {
  if (props.resizeable) {
    const storedSize = window.localStorage.getItem(localStorageKey.value);
    if (storedSize) setSize(parseInt(storedSize));
    else setSize(props.defaultSize);
  } else {
    window.localStorage.removeItem(localStorageKey.value);
  }
  windowResizeObserver.observe(document.body);
});

onBeforeUnmount(() => {
  windowResizeObserver.unobserve(document.body);
});

defineExpose({
  setSize,
  resetSize,
});
</script>
