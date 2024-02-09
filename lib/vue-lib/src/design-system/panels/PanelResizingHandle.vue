<template>
  <div
    :class="
      clsx(
        'si-panel-resizer',
        'absolute',
        isHandleVisible ? 'z-40' : 'z-30',
        panelIsVertical
          ? ['h-full', isHandleVisible ? 'w-[6px]' : 'w-[3px]']
          : ['w-full', isHandleVisible ? 'h-[6px]' : 'h-[3px]'],
        collapsed
          ? 'bg-transparent'
          : isResizing
          ? 'bg-action-300 dark:bg-action-500'
          : isHovered
          ? 'bg-action-500 dark:bg-action-300'
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
      v-if="!collapsed"
      :class="
        clsx(
          'si-panel-resizer__hover-area',
          'absolute z-50',
          showResizeHoverAreas
            ? 'bg-destructive-500 opacity-25'
            : 'bg-transparent',
          panelIsVertical
            ? ['h-full cursor-col-resize', isHandleVisible ? 'w-8' : 'w-1']
            : ['w-full cursor-row-resize', isHandleVisible ? 'h-8' : 'h-1'],
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
          'absolute shadow-md cursor-pointer z-50',
          panelSide === 'left' && 'top-1/2 translate-y-[-50%] rounded-r',
          panelSide === 'right' &&
            'top-1/2 left-full translate-y-[-50%] translate-x-[-100%]  rounded-l',
          panelSide === 'top' &&
            'left-1/2 top-full translate-x-[-50%] translate-y-[-100%] rounded-l',
          panelSide === 'bottom' && 'left-1/2 translate-x-[-50%] rounded-r',
          'w-7 h-7 bg-neutral-100 dark:bg-neutral-800 hover:bg-action-200 dark:hover:bg-action-900',
          'border border-action-500 dark:border-action-300',
          !panelIsVertical && 'rotate-90',
          !isHandleVisible && !collapsed && 'hidden',
        )
      "
      @mouseover="isHovered = true"
      @mouseleave="isHovered = false"
      @click="onClickCollapse"
    >
      <Icon
        name="double-arrow-left"
        :class="
          clsx(
            'absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-6 text-action-500 dark:text-action-300',
            (panelSide === 'right' || panelSide === 'top') &&
              !collapsed &&
              'rotate-180',
            (panelSide === 'left' || panelSide === 'bottom') &&
              collapsed &&
              'rotate-180',
          )
        "
        size="lg"
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
  collapsed: { type: Boolean },
});

const panelIsVertical = computed(() => {
  return props.panelSide === "left" || props.panelSide === "right";
});

const emit = defineEmits<{
  (e: "resize-start"): void;
  (e: "resize-end"): void;
  (e: "resize-move", delta: number): void;
  (e: "resize-reset"): void;
  (e: "collapse-close"): void;
  (e: "collapse-open"): void;
  (e: "collapse-toggle"): void;
}>();

const dragStartMouseX = ref(0);
const dragStartMouseY = ref(0);
const isHovered = ref(false);
const isResizing = ref(false);

const isHandleVisible = computed(
  () => isHovered.value || isResizing.value || props.collapsed,
);

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

const onClickCollapse = () => {
  emit("collapse-toggle");
};

onBeforeUnmount(() => {
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
});
</script>
