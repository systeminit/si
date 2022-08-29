<template>
  <div :class="classes">
    <div
      :class="hoverClasses"
      @mouseover="showHandle"
      @mouseleave="hideHandle"
      @mousedown="mouseDown"
      @dblclick="dblClick"
    />
    <div ref="handleRef" :class="handleClasses">
      <DotsHorizontalIcon v-if="isVertical" :class="iconClasses" />
      <DotsVerticalIcon v-else :class="iconClasses" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { DotsVerticalIcon, DotsHorizontalIcon } from "@heroicons/vue/solid";
import { computed, onBeforeUnmount, ref } from "vue";

const props = withDefaults(
  defineProps<{
    panelSide: "left" | "right" | "top" | "bottom";
    showResizeHoverAreas?: boolean;
  }>(),
  {
    showResizeHoverAreas: false,
  },
);

const isVertical = computed(() => {
  return props.panelSide === "top" || props.panelSide === "bottom";
});

const classes = computed(() => {
  const everyDirection = "bg-neutral-300 dark:bg-neutral-600 z-30 absolute ";

  const horizontal = "w-0.5 h-full cursor-col-resize ";

  const vertical = "h-0.5 w-full cursor-row-resize ";

  const side = { left: "right-0", right: "", top: "bottom-0", bottom: "" }[
    props.panelSide
  ];

  return everyDirection + (isVertical.value ? vertical : horizontal) + side;
});

const hoverClasses = computed(() => {
  const c = `absolute left-1/2 top-1/2 z-50 ${
    props.showResizeHoverAreas
      ? "bg-destructive-500 opacity-25"
      : "bg-transparent"
  }`;
  if (props.panelSide === "bottom")
    return `${c} w-full h-6 translate-x-[-50%] translate-y-[-100%]`;
  else if (props.panelSide === "top")
    return `${c} w-full h-6 translate-x-[-50%]`;
  else if (props.panelSide === "left")
    return `${c} w-6 h-full translate-y-[-50%]`;
  return `${c} w-6 h-full translate-y-[-50%] translate-x-[-25%]`;
});

const handleClasses = computed(() => {
  const c =
    "absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] hidden rounded-full bg-neutral-200 dark:bg-neutral-700 border border-neutral-300 dark:border-neutral-900 text-xl";
  if (isVertical.value) return `${c} h-3 w-16`;
  return `${c} w-3 h-16`;
});

const iconClasses =
  "absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-6 text-neutral-400 dark:text-neutral-500";

const emit = defineEmits<{
  (e: "resize-start"): void;
  (e: "resize-end"): void;
  (e: "resize-move", delta: number): void;
  (e: "resize-reset"): void;
}>();

const selected = ref(false);
const handleRef = ref();
const handleShow = ref(false);
const dragStartMouseX = ref(0);
const dragStartMouseY = ref(0);

const showHandle = () => {
  handleShow.value = true;
  handleRef.value.classList.remove("hidden");
};

const hideHandle = () => {
  handleShow.value = false;
  if (!selected.value) handleRef.value.classList.add("hidden");
};

const mouseDown = (e: MouseEvent) => {
  selected.value = true;
  dragStartMouseX.value = e.clientX;
  dragStartMouseY.value = e.clientY;
  window.addEventListener("mousemove", mouseMove);
  window.addEventListener("mouseup", mouseUp);
  emit("resize-start");
};

const mouseMove = (e: MouseEvent) => {
  const dx = dragStartMouseX.value - e.clientX;
  const dy = dragStartMouseY.value - e.clientY;
  emit("resize-move", isVertical.value ? dy : dx);
};

const mouseUp = () => {
  selected.value = false;
  window.removeEventListener("mousemove", mouseMove);
  window.removeEventListener("mouseup", mouseUp);
  emit("resize-end");
  if (!handleShow.value) hideHandle();
};

const dblClick = () => {
  emit("resize-reset");
};

onBeforeUnmount(() => {
  window.removeEventListener("mousemove", mouseMove);
  window.removeEventListener("mouseup", mouseUp);
});
</script>
