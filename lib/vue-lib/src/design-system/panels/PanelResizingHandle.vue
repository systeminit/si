<template>
  <div
    :class="
      clsx(
        'si-panel-resizer',
        'absolute',
        isHandleVisible ? 'z-40' : 'z-30',
        panelIsVertical
          ? [
              'h-full cursor-col-resize',
              isHandleVisible ? 'w-[4px]' : 'w-[3px]',
            ]
          : [
              'w-full cursor-row-resize',
              isHandleVisible ? 'h-[4px]' : 'h-[3px]',
            ],
        isHandleVisible
          ? 'bg-neutral-400 dark:bg-neutral-500'
          : 'bg-neutral-300 dark:bg-neutral-600',
        {
          left: 'left-full',
          right: 'right-full',
          top: 'top-full',
          bottom: 'bottom-full',
        }[panelSide],
      )
    "
  >
    <div
      :class="
        clsx(
          'si-panel-resizer__hover-area',
          'absolute z-50',
          showResizeHoverAreas
            ? 'bg-destructive-500 opacity-25'
            : 'bg-transparent',
          panelIsVertical
            ? ['h-full', isHandleVisible ? 'w-8' : 'w-1']
            : ['w-full', isHandleVisible ? 'h-8' : 'h-1'],
          {
            left: 'left-0.5 translate-x-[-50%]',
            right: 'right-0.5 translate-x-[50%]',
            top: 'top-0.5 translate-y-[-50%]',
            bottom: 'bottom-0.5 translate-y-[50%]',
          }[panelSide],
        )
      "
      @mouseover="isHovered = true"
      @mouseleave="isHovered = false"
      @mousedown="onMouseDown"
      @dblclick="onDblClick"
    />
    <div
      ref="handleRef"
      :class="
        clsx(
          'absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] shadow-md',
          'w-3 h-16 rounded-full',
          'bg-neutral-200 dark:bg-neutral-700 border border-neutral-300 dark:border-neutral-900',
          !panelIsVertical && 'rotate-90',
          !isHandleVisible && 'hidden',
        )
      "
    >
      <Icon
        name="dots-vertical"
        class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-6 text-neutral-400 dark:text-neutral-500"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, PropType, ref } from "vue";
import clsx from "clsx";
import Icon from "../icons/Icon.vue";

const props = defineProps({
  panelSide: {
    type: String as PropType<"left" | "right" | "top" | "bottom">,
    required: true,
  },
  showResizeHoverAreas: { type: Boolean },
});

const panelIsVertical = computed(() => {
  return props.panelSide === "left" || props.panelSide === "right";
});

const emit = defineEmits<{
  (e: "resize-start"): void;
  (e: "resize-end"): void;
  (e: "resize-move", delta: number): void;
  (e: "resize-reset"): void;
}>();

const dragStartMouseX = ref(0);
const dragStartMouseY = ref(0);
const isHovered = ref(false);
const isResizing = ref(false);

const isHandleVisible = computed(() => isHovered.value || isResizing.value);

const onMouseDown = (e: MouseEvent) => {
  isResizing.value = true;
  dragStartMouseX.value = e.clientX;
  dragStartMouseY.value = e.clientY;
  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
  emit("resize-start");
};

const onMouseMove = (e: MouseEvent) => {
  const dx = dragStartMouseX.value - e.clientX;
  const dy = dragStartMouseY.value - e.clientY;
  emit("resize-move", panelIsVertical.value ? dx : dy);
};

const onMouseUp = () => {
  isResizing.value = false;
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
  emit("resize-end");
};

const onDblClick = () => {
  emit("resize-reset");
};

onBeforeUnmount(() => {
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
});
</script>
