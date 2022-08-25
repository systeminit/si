<template>
  <div ref="panel" :class="panelClasses">
    <SiPanelResizer
      v-if="currentlyResizeable"
      :panel-side="side"
      @start-resize="startResize"
      @resizing="resizing"
      @reset-size="resetSize"
    />
    <div class="absolute w-full h-full flex flex-col">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, toRef } from "vue";
import SiPanelResizer from "@/atoms/SiPanelResizer.vue";
import _ from "lodash";

const props = withDefaults(
  defineProps<{
    id: string;
    side: "left" | "right" | "top" | "bottom";
    hidden?: boolean;
    classes?: string;
    sizeClasses?: string;
    resizeable?: boolean;
    minResize?: number;
    maxResize?: number;
    fixedDefaultSize?: number | undefined;
  }>(),
  {
    classes:
      "z-20 dark:text-white border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 pointer-events-auto relative",
    sizeClasses: "",
    resizeable: true,
    minResize: 0.1,
    maxResize: 0.45,
    fixedDefaultSize: undefined,
  },
);
const side = toRef(props, "side");
const isVertical = computed(
  () => props.side === "top" || props.side === "bottom",
);

const panelClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (side.value == "left") {
    if (!currentlyResizeable.value) classes["border-r-2"] = true;
    classes[props.hidden ? "right-96" : "right-0"] = true;
  } else if (side.value == "right") {
    if (!currentlyResizeable.value) classes["border-l-2"] = true;
    classes[props.hidden ? "left-96" : "left-0"] = true;
  } else if (side.value == "top") {
    if (!currentlyResizeable.value) classes["border-b-2"] = true;
    classes[props.hidden ? "top-96" : "top-0"] = true;
  } else if (side.value == "bottom") {
    if (!currentlyResizeable.value) classes["border-t-2"] = true;
    classes[props.hidden ? "bottom-96" : "bottom-0"] = true;
  }
  const propClasses = props.classes
    .split(" ")
    .concat(props.sizeClasses.split(" "));

  propClasses.forEach((c) => {
    classes[c] = true;
  });

  return classes;
});

const panel = ref();
const size = ref(0);
const maxResize = toRef(props, "maxResize");
const currentlyResizeable = ref(props.resizeable);
const currentMinResize = ref(props.minResize);

const setCurrentlyResizeable = (v: boolean) => {
  currentlyResizeable.value = v;
};

const setCurrentMinResize = (v: number) => {
  currentMinResize.value = v;
};

const setSize = (size: number, delta = 0) => {
  let finalSize =
    size + (side.value === "right" || side.value === "bottom" ? delta : -delta);

  if (currentMinResize.value > 1) {
    if (finalSize < currentMinResize.value) finalSize = currentMinResize.value;
  } else if (currentMinResize.value > 0) {
    const limit =
      (isVertical.value ? window.innerHeight : window.innerWidth) *
      currentMinResize.value;
    if (finalSize < limit) finalSize = limit;
  }
  if (maxResize.value > 1) {
    if (finalSize > maxResize.value) finalSize = maxResize.value;
  } else if (maxResize.value > 0) {
    const limit =
      (isVertical.value ? window.innerHeight : window.innerWidth) *
      maxResize.value;

    if (finalSize > limit) finalSize = limit;
  }

  if (isVertical.value) panel.value.style.height = finalSize + "px";
  else panel.value.style.width = finalSize + "px";
  return finalSize;
};

const startResize = () => {
  if (isVertical.value) size.value = panel.value.offsetHeight;
  else size.value = panel.value.offsetWidth;
};

const resizing = (delta: number) => {
  const finalSize = setSize(size.value, delta);
  window.localStorage.setItem(props.id + "-size", "" + finalSize);
};

const resetSize = (useDefaultSize = true) => {
  window.localStorage.removeItem(props.id + "-size");
  panel.value.style.width = "";
  panel.value.style.height = "";
  if (props.fixedDefaultSize && useDefaultSize) setSize(props.fixedDefaultSize);
};

const onWindowResize = () => {
  const currentSize = isVertical.value
    ? panel.value.offsetHeight
    : panel.value.offsetWidth;
  setSize(currentSize);
};

const debounceForResize = _.debounce(onWindowResize, 20);
const resizeObserver = new ResizeObserver(debounceForResize);

onMounted(() => {
  if (props.resizeable) {
    const storedSize = window.localStorage.getItem(props.id + "-size");
    if (storedSize) setSize(parseInt(storedSize), 0);
  } else {
    window.localStorage.removeItem(props.id + "-size");
  }
  resizeObserver.observe(document.body);
});

onBeforeUnmount(() => {
  resizeObserver.unobserve(document.body);
});

defineExpose({
  setSize,
  setCurrentlyResizeable,
  resetSize,
  setCurrentMinResize,
});
</script>
