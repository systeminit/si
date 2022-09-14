<template>
  <div
    :class="
      clsx(
        'z-30 absolute',
        isVertical
          ? 'h-0.5 w-full cursor-row-resize'
          : 'w-0.5 h-full cursor-col-resize',
        {
          left: '-right-0.5',
          right: '-left-0.5',
          top: '-bottom-0.5',
          bottom: '-top-0.5',
        }[panelSide],
      )
    "
  >
    <div
      :class="
        clsx(
          'absolute z-50',
          showResizeHoverAreas
            ? 'bg-destructive-500 opacity-25'
            : 'bg-transparent',
          isVertical ? 'h-5 w-full' : 'w-5 h-full',
          {
            left: 'left-0',
            right: 'right-0',
            top: 'top-0',
            bottom: 'bottom-0',
          }[panelSide],
        )
      "
      @mouseover="showHandle"
      @mouseleave="hideHandle"
      @mousedown="mouseDown"
      @dblclick="dblClick"
    />
    <div
      ref="handleRef"
      :class="
        clsx(
          'absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] rounded-full bg-neutral-200 dark:bg-neutral-700 border border-neutral-300 dark:border-neutral-900 text-xl',
          isVertical ? 'h-3 w-16' : 'w-3 h-16',
          !handleVisible && 'hidden',
        )
      "
    >
      <Icon
        :name="isVertical ? 'dots-horizontal' : 'dots-vertical'"
        class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-6 text-neutral-400 dark:text-neutral-500"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, PropType, ref } from "vue";
import clsx from "clsx";
import Icon from "@/ui-lib/Icon.vue";

const props = defineProps({
  panelSide: {
    type: String as PropType<"left" | "right" | "top" | "bottom">,
    required: true,
  },
  showResizeHoverAreas: { type: Boolean },
});

const isVertical = computed(() => {
  return props.panelSide === "top" || props.panelSide === "bottom";
});

const emit = defineEmits<{
  (e: "resize-start"): void;
  (e: "resize-end"): void;
  (e: "resize-move", delta: number): void;
  (e: "resize-reset"): void;
}>();

const selected = ref(false);
const dragStartMouseX = ref(0);
const dragStartMouseY = ref(0);
const handleVisible = ref(false);

const showHandle = () => {
  handleVisible.value = true;
};

const hideHandle = () => {
  handleVisible.value = false;
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
  if (!handleVisible.value) hideHandle();
};

const dblClick = () => {
  emit("resize-reset");
};

onBeforeUnmount(() => {
  window.removeEventListener("mousemove", mouseMove);
  window.removeEventListener("mouseup", mouseUp);
});
</script>
