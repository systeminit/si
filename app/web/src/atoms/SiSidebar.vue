<template>
  <!-- FIXME(nick,victor): find a way to remove z levels from here. This is needed for interaction with the canvas to work. -->
  <SiPanelResizer v-if="resizeable && side === 'right'" />
  <div :class="panelClasses">
    <slot />
  </div>
  <SiPanelResizer v-if="resizeable && side === 'left'" />
</template>

<script setup lang="ts">
import { computed, toRef } from "vue";
import SiPanelResizer from "@/atoms/SiPanelResizer.vue";

const props = withDefaults(
  defineProps<{
    side: "right" | "left";
    hidden?: boolean;
    classes?: string;
    widthClasses?: string;
    resizeable: boolean;
  }>(),
  {
    classes:
      "z-20 flex flex-col dark:text-white border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 pointer-events-auto relative transition-all",
    widthClasses: "w-72 xl:w-96",
    resizeable: true,
  },
);
const side = toRef(props, "side");

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
</script>

<style>
/* TODO(WENDY) - Why are these Tailwind classes not working? Had to add them manually. */
.cursor-col-resize {
  cursor: col-resize;
}
</style>
