<template>
  <div ref="panel" :class="panelClasses">
    <SiPanelResizer
      v-if="resizeable"
      :panel-side="side"
      @start-resize="startResize"
      @resizing="resizing"
      @reset-size="resetSize"
    />
    <div class="absolute w-full h-full">
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
    widthClasses?: string;
    resizeable?: boolean;
    minResize?: number;
    maxResize?: number;
  }>(),
  {
    classes:
      "z-20 flex flex-col dark:text-white border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 pointer-events-auto relative", // transition-all
    widthClasses: "w-72 xl:w-96",
    resizeable: true,
    minResize: 0.1,
    maxResize: 0.45,
  },
);
const side = toRef(props, "side");
const isVertical = computed(
  () => props.side === "top" || props.side === "bottom",
);

const panelClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (side.value == "left") {
    if (!props.resizeable) classes["border-r-2"] = true;
    classes[props.hidden ? "right-96" : "right-0"] = true;
  } else {
    if (!props.resizeable) classes["border-l-2"] = true;
    classes[props.hidden ? "left-96" : "left-0"] = true;
  }
  const propClasses = props.classes
    .split(" ")
    .concat(props.widthClasses.split(" "));

  propClasses.forEach((c) => {
    classes[c] = true;
  });

  return classes;
});

const panel = ref();
const size = ref(0);
const maxResize = toRef(props, "maxResize");
const minResize = toRef(props, "minResize");

const setSize = (size: number, delta: number) => {
  let finalSize =
    size + (side.value === "right" || side.value === "bottom" ? delta : -delta);

  if (minResize.value > 1) {
    if (finalSize < minResize.value) finalSize = minResize.value;
  } else if (minResize.value > 0) {
    const limit =
      (isVertical.value ? window.innerHeight : window.innerWidth) *
      minResize.value;
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

  panel.value.style.width = finalSize + "px";
  return finalSize;
};

const startResize = () => {
  size.value = panel.value.clientWidth;
};

const resizing = (delta: number) => {
  const finalSize = setSize(size.value, delta);
  window.localStorage.setItem(props.id + "-size", "" + finalSize);
};

const resetSize = () => {
  window.localStorage.removeItem(props.id + "-size");
  panel.value.style.width = "";
};

const onWindowResize = () => {
  const currentSize = panel.value.clientWidth;
  setSize(currentSize, 0);
};

const debounceForResize = _.debounce(onWindowResize, 20);
const resizeObserver = new ResizeObserver(debounceForResize);

onMounted(() => {
  const storedSize = window.localStorage.getItem(props.id + "-size");
  if (storedSize) setSize(parseInt(storedSize), 0);

  resizeObserver.observe(document.body);
});

onBeforeUnmount(() => {
  resizeObserver.unobserve(document.body);
});
</script>
