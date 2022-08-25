<template>
  <div :class="classes" @mousedown="mouseDown" @dblclick="dblClick">
    <div
      class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] bg-transparent w-10 h-full z-50"
      @mouseover="showHandle"
      @mouseleave="hideHandle"
    ></div>
    <div
      ref="handle"
      class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] hidden w-3 h-16 rounded-full bg-neutral-200 dark:bg-neutral-700 border border-neutral-300 dark:border-neutral-900 text-xl"
    >
      <DotsVerticalIcon
        class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-6 text-neutral-400 dark:text-neutral-500"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { DotsVerticalIcon } from "@heroicons/vue/solid";
import { computed, defineEmits, ref } from "vue";

const props = defineProps<{
  panelSide: "left" | "right" | "top" | "bottom";
}>();

const isVertical = computed(() => {
  return props.panelSide === "top" || props.panelSide === "bottom";
});

const classes = computed(() => {
  const everyDirection = "bg-neutral-300 dark:bg-neutral-600 z-30 absolute ";

  const horizontal = "w-0.5 h-full cursor-col-resize ";

  const vertical = "h-0.5 w-full cursor-row-resize ";

  const side = { left: "right-0", right: "", top: "", bottom: "" }[
    props.panelSide
  ];

  return everyDirection + (isVertical.value ? vertical : horizontal) + side;
});

const emit = defineEmits<{
  (e: "start-resize"): void;
  (e: "end-resize"): void;
  (e: "resizing", delta: number): void;
  (e: "reset-size"): void;
}>();

const selected = ref(false);
const handle = ref();
const handleShow = ref(false);
const oldMouseX = ref(0);
const oldMouseY = ref(0);

const showHandle = () => {
  handleShow.value = true;
  handle.value.classList.remove("hidden");
};

const hideHandle = () => {
  handleShow.value = false;
  if (!selected.value) handle.value.classList.add("hidden");
};

const mouseDown = (e: MouseEvent) => {
  selected.value = true;
  oldMouseX.value = e.clientX;
  oldMouseY.value = e.clientY;
  window.addEventListener("mousemove", mouseMove);
  window.addEventListener("mouseup", mouseUp);
  emit("start-resize");
};

const mouseMove = (e: MouseEvent) => {
  const dx = oldMouseX.value - e.clientX;
  const dy = oldMouseY.value - e.clientY;
  emit("resizing", isVertical.value ? dy : dx);
};

const mouseUp = () => {
  selected.value = false;
  window.removeEventListener("mousemove", mouseMove);
  window.removeEventListener("mouseup", mouseUp);
  emit("end-resize");
  if (!handleShow.value) hideHandle();
};

const dblClick = () => {
  emit("reset-size");
};
</script>
